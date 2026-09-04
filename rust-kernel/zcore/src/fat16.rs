//! Порт `kernel/fat16.c` + `fat16.h` — файловая система FAT16 (RAM-диск/ATA).

use core::ptr;

use zlibk::ctype::toupper;
use zlibk::mem::{memcmp, memcpy, memset};
use zlibk::str::{strcmp, strlen};

use crate::drivers::ata::{ata_read_sector, ata_write_sector};
use crate::drivers::video::{
    video_print, video_print_char, video_put_dec, video_set_color, COLOR_BLACK, COLOR_DARK_GREY,
    COLOR_LIGHT_CYAN, COLOR_LIGHT_GREEN, COLOR_WHITE, COLOR_YELLOW,
};

const SECTOR_SIZE: usize = 512;
const FAT16_PARTITION_START: u32 = 2048;

/// BPB из загрузочного сектора FAT16 (`fat16_bpb_t`).
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct Fat16Bpb {
    jmp: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    root_entries: u16,
    total_sectors_16: u16,
    media_type: u8,
    fat_size_16: u16,
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    drive_number: u8,
    reserved1: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
    pad: [u8; 450],
}

/// Запись директории FAT16 (`fat16_dir_entry_t`, 32 байта).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Fat16DirEntry {
    pub name: [u8; 8],
    pub ext: [u8; 3],
    pub attributes: u8,
    pub reserved: u8,
    pub create_time_ms: u8,
    pub create_time: u16,
    pub create_date: u16,
    pub access_date: u16,
    pub cluster_high: u16,
    pub modify_time: u16,
    pub modify_date: u16,
    pub cluster_low: u16,
    pub file_size: u32,
}

static mut BPB: Fat16Bpb = Fat16Bpb {
    jmp: [0; 3],
    oem_name: [0; 8],
    bytes_per_sector: 0,
    sectors_per_cluster: 0,
    reserved_sectors: 0,
    num_fats: 0,
    root_entries: 0,
    total_sectors_16: 0,
    media_type: 0,
    fat_size_16: 0,
    sectors_per_track: 0,
    num_heads: 0,
    hidden_sectors: 0,
    total_sectors_32: 0,
    drive_number: 0,
    reserved1: 0,
    boot_signature: 0,
    volume_id: 0,
    volume_label: [0; 11],
    fs_type: [0; 8],
    pad: [0; 450],
};
static mut FAT_START: u32 = 0;
static mut ROOT_DIR_START: u32 = 0;
static mut DATA_START: u32 = 0;
static mut ROOT_DIR_SECTORS: u32 = 0;
static mut SEC_BUF: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
static mut SEC_BUF2: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
static mut FAT_BUF: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
static mut CACHED_FAT_SEC: u32 = 0xFFFFFFFF;
static mut FAT_DIRTY: i32 = 0;

fn rd(lba: u32, b: *mut u8) -> i32 {
    ata_read_sector(FAT16_PARTITION_START + lba, b)
}

fn wr(lba: u32, b: *const u8) -> i32 {
    ata_write_sector(FAT16_PARTITION_START + lba, b)
}

fn clus_lba(c: u16) -> u32 {
    unsafe { DATA_START + (c as u32 - 2) * (BPB.sectors_per_cluster as u32) }
}

// FAT cache

fn fat_get(c: u16) -> u16 {
    unsafe {
        let sec = ((c as u32) * 2) / SECTOR_SIZE as u32;
        let off = (((c as u32) * 2) % SECTOR_SIZE as u32) as usize;
        if sec != CACHED_FAT_SEC {
            if FAT_DIRTY != 0 {
                wr(FAT_START + CACHED_FAT_SEC, FAT_BUF.as_ptr());
                FAT_DIRTY = 0;
            }
            rd(FAT_START + sec, FAT_BUF.as_mut_ptr());
            CACHED_FAT_SEC = sec;
        }
        ptr::read_unaligned(FAT_BUF.as_ptr().add(off) as *const u16)
    }
}

fn fat_set(c: u16, v: u16) {
    unsafe {
        let sec = ((c as u32) * 2) / SECTOR_SIZE as u32;
        let off = (((c as u32) * 2) % SECTOR_SIZE as u32) as usize;
        if sec != CACHED_FAT_SEC {
            if FAT_DIRTY != 0 {
                wr(FAT_START + CACHED_FAT_SEC, FAT_BUF.as_ptr());
                FAT_DIRTY = 0;
            }
            rd(FAT_START + sec, FAT_BUF.as_mut_ptr());
            CACHED_FAT_SEC = sec;
        }
        ptr::write_unaligned(FAT_BUF.as_mut_ptr().add(off) as *mut u16, v);
        FAT_DIRTY = 1;
    }
}

fn fat_flush() {
    unsafe {
        if FAT_DIRTY != 0 && CACHED_FAT_SEC != 0xFFFFFFFF {
            wr(FAT_START + CACHED_FAT_SEC, FAT_BUF.as_ptr());
            wr(FAT_START + BPB.fat_size_16 as u32 + CACHED_FAT_SEC, FAT_BUF.as_ptr());
            FAT_DIRTY = 0;
        }
    }
}

fn fat_alloc() -> u16 {
    unsafe {
        let total = if BPB.total_sectors_16 != 0 {
            BPB.total_sectors_16 as u32
        } else {
            BPB.total_sectors_32
        };
        let max_c = ((total - DATA_START) / (BPB.sectors_per_cluster as u32) + 2) as u16;
        let mut c: u16 = 2;
        while c < max_c {
            if fat_get(c) == 0x0000 {
                fat_set(c, 0xFFFF);
                return c;
            }
            c += 1;
        }
        0
    }
}

// Имена

fn to83(fn_: *const u8, out: *mut u8) {
    unsafe {
        memset(out, b' ' as i32, 11);
        let mut i: i32 = 0;
        let mut j: i32 = 0;
        while *fn_.add(i as usize) != 0 && *fn_.add(i as usize) != b'.' && j < 8 {
            out.add(j as usize).write(toupper(*fn_.add(i as usize) as i32) as u8);
            i += 1;
            j += 1;
        }
        while *fn_.add(i as usize) != 0 && *fn_.add(i as usize) != b'.' {
            i += 1;
        }
        if *fn_.add(i as usize) == b'.' {
            i += 1;
            j = 8;
            while *fn_.add(i as usize) != 0 && j < 11 {
                out.add(j as usize).write(toupper(*fn_.add(i as usize) as i32) as u8);
                i += 1;
                j += 1;
            }
        }
    }
}

fn from83(in_: *const u8, out: *mut u8) {
    unsafe {
        let mut j: i32 = 0;
        for i in 0..8usize {
            if *in_.add(i) == b' ' {
                break;
            }
            out.add(j as usize).write(*in_.add(i));
            j += 1;
        }
        if *in_.add(8) != b' ' {
            out.add(j as usize).write(b'.');
            j += 1;
            for i in 8..11usize {
                if *in_.add(i) == b' ' {
                    break;
                }
                out.add(j as usize).write(*in_.add(i));
                j += 1;
            }
        }
        out.add(j as usize).write(0);
    }
}

// Инициализация

pub fn fat16_init(_ps: u32) -> i32 {
    unsafe {
        if rd(0, &mut BPB as *mut Fat16Bpb as *mut u8) != 0 {
            return -1;
        }
        if BPB.bytes_per_sector != 512 {
            return -1;
        }
        FAT_START = BPB.reserved_sectors as u32;
        ROOT_DIR_SECTORS = ((BPB.root_entries as u32 * 32) + 511) / 512;
        ROOT_DIR_START = FAT_START + BPB.num_fats as u32 * BPB.fat_size_16 as u32;
        DATA_START = ROOT_DIR_START + ROOT_DIR_SECTORS;
        0
    }
}

// Поиск файла по корневой директории

pub fn fat16_find_file(filename: *const u8, entry: *mut Fat16DirEntry) -> i32 {
    unsafe {
        // Есть ли слэш?
        let mut sl = filename;
        while *sl != 0 && *sl != b'/' {
            sl = sl.add(1);
        }

        if *sl == b'/' {
            let mut d83 = [b' '; 11];
            let dlen = (sl as usize - filename as usize).min(8);
            for i in 0..dlen {
                d83[i] = toupper(*filename.add(i) as i32) as u8;
            }

            let mut de: Fat16DirEntry = core::mem::zeroed();
            let mut found = 0;
            let mut s: u32 = 0;
            while s < ROOT_DIR_SECTORS && found == 0 {
                if rd(ROOT_DIR_START + s, SEC_BUF.as_mut_ptr()) != 0 {
                    return -1;
                }
                let mut e: usize = 0;
                while e < SECTOR_SIZE / 32 {
                    let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                    let e_ = &*eptr.add(e);
                    if e_.name[0] == 0x00 {
                        break;
                    }
                    if e_.name[0] == 0xE5 || e_.attributes & 0x10 == 0 {
                        e += 1;
                        continue;
                    }
                    if memcmp(e_.name.as_ptr(), d83.as_ptr(), 8) == 0 {
                        de = *e_;
                        found = 1;
                        break;
                    }
                    e += 1;
                }
                s += 1;
            }
            if found == 0 {
                return -1;
            }

            let mut f83 = [0u8; 11];
            to83(sl.add(1), f83.as_mut_ptr());
            let mut clus = de.cluster_low;
            while clus < 0xFFF8 {
                let lba = clus_lba(clus);
                for s in 0..BPB.sectors_per_cluster {
                    if rd(lba + s as u32, SEC_BUF.as_mut_ptr()) != 0 {
                        return -1;
                    }
                    for i in 0..SECTOR_SIZE / 32 {
                        let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                        let e_ = &*eptr.add(i);
                        if e_.name[0] == 0x00 {
                            return -1;
                        }
                        if e_.name[0] == 0xE5 || e_.attributes == 0x0F || e_.attributes & 0x08 != 0 {
                            continue;
                        }
                        if memcmp(e_.name.as_ptr(), f83.as_ptr(), 11) == 0 {
                            *entry = *e_;
                            return 0;
                        }
                    }
                }
                clus = fat_get(clus);
            }
            return -1;
        }

        // Корневая директория
        let mut n83 = [0u8; 11];
        to83(filename, n83.as_mut_ptr());
        let mut s: u32 = 0;
        while s < ROOT_DIR_SECTORS {
            if rd(ROOT_DIR_START + s, SEC_BUF.as_mut_ptr()) != 0 {
                return -1;
            }
            for i in 0..SECTOR_SIZE / 32 {
                let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                let e_ = &*eptr.add(i);
                if e_.name[0] == 0x00 {
                    return -1;
                }
                if e_.name[0] == 0xE5 || e_.attributes == 0x0F || e_.attributes & 0x08 != 0 {
                    continue;
                }
                if memcmp(e_.name.as_ptr(), n83.as_ptr(), 11) == 0 {
                    *entry = *e_;
                    return 0;
                }
            }
            s += 1;
        }
        -1
    }
}

// Чтение

pub fn fat16_read_file(filename: *const u8, buffer: *mut u8, max_size: u32) -> i32 {
    unsafe {
        let mut e: Fat16DirEntry = core::mem::zeroed();
        if fat16_find_file(filename, &mut e) != 0 {
            return -1;
        }
        let buf = buffer;
        let mut rd_total: u32 = 0;
        let mut fsz = e.file_size;
        if fsz > max_size {
            fsz = max_size;
        }
        let mut clus = e.cluster_low;
        while clus < 0xFFF8 && rd_total < fsz {
            let lba = clus_lba(clus);
            for s in 0..BPB.sectors_per_cluster {
                if rd(lba + s as u32, SEC_BUF.as_mut_ptr()) != 0 {
                    return -1;
                }
                let mut tc: u32 = SECTOR_SIZE as u32;
                if rd_total + tc > fsz {
                    tc = fsz - rd_total;
                }
                memcpy(buf.add(rd_total as usize), SEC_BUF.as_ptr(), tc as usize);
                rd_total += tc;
                if rd_total >= fsz {
                    return rd_total as i32;
                }
            }
            clus = fat_get(clus);
        }
        rd_total as i32
    }
}

// Запись / создание файла

pub fn fat16_write_file(mut filename: *const u8, data: *const u8, size: u32) -> i32 {
    unsafe {
        let mut sl = filename;
        while *sl != 0 && *sl != b'/' {
            sl = sl.add(1);
        }

        let in_subdir = *sl == b'/';

        let mut dir_clus: u16 = 0; // 0 = корень

        if in_subdir {
            // Поиск поддиректории
            let mut d83 = [b' '; 11];
            let dlen = (sl as usize - filename as usize).min(8);
            for i in 0..dlen {
                d83[i] = toupper(*filename.add(i) as i32) as u8;
            }

            let mut de: Fat16DirEntry = core::mem::zeroed();
            let mut found = 0;
            let mut s: u32 = 0;
            while s < ROOT_DIR_SECTORS && found == 0 {
                if rd(ROOT_DIR_START + s, SEC_BUF.as_mut_ptr()) != 0 {
                    return -1;
                }
                let mut i: usize = 0;
                while i < SECTOR_SIZE / 32 {
                    let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                    let e_ = &*eptr.add(i);
                    if e_.name[0] == 0x00 {
                        i = SECTOR_SIZE / 32;
                        s = ROOT_DIR_SECTORS;
                        break;
                    }
                    if e_.name[0] == 0xE5 {
                        i += 1;
                        continue;
                    }
                    if e_.attributes & 0x10 == 0 {
                        i += 1;
                        continue;
                    }
                    if memcmp(e_.name.as_ptr(), d83.as_ptr(), 8) == 0 {
                        de = *e_;
                        found = 1;
                        break;
                    }
                    i += 1;
                }
                s += 1;
            }
            if found == 0 {
                return -1;
            }

            dir_clus = de.cluster_low;
            filename = sl.add(1); // имя файла без пути
        }

        let mut n83 = [0u8; 11];
        to83(filename, n83.as_mut_ptr());

        // Ищем существующую или свободную запись в нужной директории
        let mut esec: u32 = 0;
        let mut eidx: usize = 0;
        let mut efound: i32 = 0;
        let mut old: Fat16DirEntry = core::mem::zeroed();

        if !in_subdir {
            // Корневая директория
            let mut s: u32 = 0;
            'outer: while s < ROOT_DIR_SECTORS && efound == 0 {
                if rd(ROOT_DIR_START + s, SEC_BUF.as_mut_ptr()) != 0 {
                    return -1;
                }
                for i in 0..SECTOR_SIZE / 32 {
                    let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                    let e_ = &*eptr.add(i);
                    let f = e_.name[0];
                    if efound == 0 && (f == 0x00 || f == 0xE5) {
                        esec = s;
                        eidx = i;
                        efound = 1;
                        break 'outer;
                    }
                    if memcmp(e_.name.as_ptr(), n83.as_ptr(), 11) == 0 {
                        esec = s;
                        eidx = i;
                        old = *e_;
                        efound = 2;
                        break 'outer;
                    }
                }
                s += 1;
            }
        } else {
            // Поддиректория: поиск по кластерам
            let mut clus = dir_clus;
            'outer: while clus < 0xFFF8 && efound == 0 {
                let lba = clus_lba(clus);
                for s in 0..BPB.sectors_per_cluster {
                    if efound != 0 {
                        break 'outer;
                    }
                    if rd(lba + s as u32, SEC_BUF.as_mut_ptr()) != 0 {
                        return -1;
                    }
                    for i in 0..SECTOR_SIZE / 32 {
                        let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                        let e_ = &*eptr.add(i);
                        let f = e_.name[0];
                        if efound == 0 && (f == 0x00 || f == 0xE5) {
                            esec = lba + s as u32;
                            eidx = i;
                            efound = 1;
                            break 'outer;
                        }
                        if memcmp(e_.name.as_ptr(), n83.as_ptr(), 11) == 0 {
                            esec = lba + s as u32;
                            eidx = i;
                            old = *e_;
                            efound = 2;
                            break 'outer;
                        }
                    }
                }
                clus = fat_get(clus);
            }
        }
        if efound == 0 {
            return -1;
        }
        // освобождение старых кластеров
        if efound == 2 && old.cluster_low != 0 {
            let mut c = old.cluster_low;
            while c < 0xFFF8 {
                let nx = fat_get(c);
                fat_set(c, 0);
                c = nx;
            }
        }

        // Выделение новых кластеров и запись на них
        let csz = BPB.sectors_per_cluster as u32 * SECTOR_SIZE as u32;
        let mut cn = (size + csz - 1) / csz;
        if cn == 0 {
            cn = 1;
        }
        let mut first_c: u16 = 0;
        let mut prev_c: u16 = 0;
        let src = data;
        let mut wrt: u32 = 0;
        for _ci in 0..cn {
            let c = fat_alloc();
            if c == 0 {
                fat_flush();
                return -1;
            }
            if prev_c != 0 {
                fat_set(prev_c, c);
            } else {
                first_c = c;
            }
            fat_set(c, 0xFFFF);
            prev_c = c;
            let lba = clus_lba(c);
            for s in 0..BPB.sectors_per_cluster {
                memset(SEC_BUF2.as_mut_ptr(), 0, SECTOR_SIZE);
                let mut tw: u32 = if wrt < size { size - wrt } else { 0 };
                if tw > SECTOR_SIZE as u32 {
                    tw = SECTOR_SIZE as u32;
                }
                if tw != 0 {
                    memcpy(SEC_BUF2.as_mut_ptr(), src.add(wrt as usize), tw as usize);
                }
                wrt += tw;
                if wr(lba + s as u32, SEC_BUF2.as_ptr()) != 0 {
                    fat_flush();
                    return -1;
                }
            }
        }
        fat_flush();

        // Запись или обновление записи директории
        let dir_sector_lba = if in_subdir {
            esec
        } else {
            ROOT_DIR_START + esec
        };
        if rd(dir_sector_lba, SEC_BUF.as_mut_ptr()) != 0 {
            return -1;
        }
        let eptr = SEC_BUF.as_mut_ptr() as *mut Fat16DirEntry;
        let e_ = &mut *eptr.add(eidx);
        *e_ = core::mem::zeroed();
        e_.name.copy_from_slice(&n83);
        e_.attributes = 0x20;
        e_.cluster_low = first_c;
        e_.file_size = size;
        if wr(dir_sector_lba, SEC_BUF.as_ptr()) != 0 {
            return -1;
        }
        size as i32
    }
}

// Создание директории

pub fn fat16_mkdir(dirname: *const u8) -> i32 {
    unsafe {
        let mut n83 = [b' '; 11];
        let mut i: usize = 0;
        while *dirname.add(i) != 0 && i < 8 {
            n83[i] = toupper(*dirname.add(i) as i32) as u8;
            i += 1;
        }

        // Проверка существования
        let mut s: u32 = 0;
        while s < ROOT_DIR_SECTORS {
            if rd(ROOT_DIR_START + s, SEC_BUF.as_mut_ptr()) != 0 {
                return -1;
            }
            let mut i2: usize = 0;
            while i2 < SECTOR_SIZE / 32 {
                let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                let e_ = &*eptr.add(i2);
                if e_.name[0] == 0x00 {
                    s = ROOT_DIR_SECTORS;
                    break;
                }
                if e_.attributes & 0x10 != 0 && memcmp(e_.name.as_ptr(), n83.as_ptr(), 8) == 0 {
                    return 0;
                }
                i2 += 1;
            }
            s += 1;
        }

        let c = fat_alloc();
        if c == 0 {
            return -1;
        }
        let lba = clus_lba(c);
        memset(SEC_BUF2.as_mut_ptr(), 0, SECTOR_SIZE);
        for s in 0..BPB.sectors_per_cluster {
            let _ = wr(lba + s as u32, SEC_BUF2.as_ptr());
        }
        fat_flush();

        let mut s3: u32 = 0;
        while s3 < ROOT_DIR_SECTORS {
            if rd(ROOT_DIR_START + s3, SEC_BUF.as_mut_ptr()) != 0 {
                return -1;
            }
            for i3 in 0..SECTOR_SIZE / 32 {
                let eptr = SEC_BUF.as_mut_ptr() as *mut Fat16DirEntry;
                let e_ = &mut *eptr.add(i3);
                let f = e_.name[0];
                if f == 0x00 || f == 0xE5 {
                    *e_ = core::mem::zeroed();
                    e_.name.copy_from_slice(&n83);
                    e_.attributes = 0x10;
                    e_.cluster_low = c;
                    if wr(ROOT_DIR_START + s3, SEC_BUF.as_ptr()) != 0 {
                        return -1;
                    }
                    return 0;
                }
            }
            s3 += 1;
        }
        -1
    }
}

// Список директории

fn list_common_entry(eidx: usize, count: &mut i32) {
    unsafe {
        let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
        let e_ = &*eptr.add(eidx);
        let mut fn_ = [0u8; 13];
        from83(e_.name.as_ptr(), fn_.as_mut_ptr());
        let fl = strlen(fn_.as_ptr());
        if fl >= 4 && strcmp(&fn_[fl - 4], b".ZXE".as_ptr()) == 0 {
            video_set_color(COLOR_LIGHT_GREEN, COLOR_BLACK);
        } else {
            video_set_color(COLOR_WHITE, COLOR_BLACK);
        }
        video_print(b"  \0".as_ptr());
        video_print(fn_.as_ptr());
        let mut p = fl;
        while p < 14 {
            video_print_char(b' ');
            p += 1;
        }
        video_set_color(COLOR_DARK_GREY, COLOR_BLACK);
        video_put_dec(e_.file_size);
        video_print(b" bytes\n\0".as_ptr());
        *count += 1;
    }
}

pub fn fat16_list_dir(path: *const u8) -> i32 {
    unsafe {
        let mut count = 0;
        video_set_color(COLOR_YELLOW, COLOR_BLACK);
        video_print(b"\n  \xD0\xA1\xD0\xBF\xD0\xB8\xD1\x81\xD0\xBE\xD0\xBA \xD0\xB4\xD0\xB8\xD1\x80\xD0\xB5\xD0\xBA\xD1\x82\xD0\xBE\xD1\x80\xD0\xB8\xD0\xB9 /\0".as_ptr());
        if !path.is_null() && *path != 0 && !(*path == b'/' && *path.add(1) == 0) {
            video_print(path);
        }
        video_print(b"\n\n\0".as_ptr());
        video_set_color(COLOR_WHITE, COLOR_BLACK);

        // Корень
        if path.is_null() || *path == 0 || (*path == b'/' && *path.add(1) == 0) {
            let mut s: u32 = 0;
            while s < ROOT_DIR_SECTORS {
                if rd(ROOT_DIR_START + s, SEC_BUF.as_mut_ptr()) != 0 {
                    return -1;
                }
                let mut i: usize = 0;
                while i < SECTOR_SIZE / 32 {
                    let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                    let e_ = &*eptr.add(i);
                    if e_.name[0] == 0x00 {
                        s = ROOT_DIR_SECTORS;
                        break;
                    }
                    if e_.name[0] == 0xE5 || e_.attributes == 0x0F || e_.attributes & 0x08 != 0 {
                        i += 1;
                        continue;
                    }
                    let mut fn_ = [0u8; 13];
                    from83(e_.name.as_ptr(), fn_.as_mut_ptr());
                    if e_.attributes & 0x10 != 0 {
                        video_set_color(COLOR_LIGHT_CYAN, COLOR_BLACK);
                        video_print(b"  \0".as_ptr());
                        video_print(fn_.as_ptr());
                        let mut p = strlen(fn_.as_ptr());
                        while p < 14 {
                            video_print_char(b' ');
                            p += 1;
                        }
                        video_print(b"<DIR>\n\0".as_ptr());
                        count += 1;
                    } else {
                        list_common_entry(i, &mut count);
                    }
                    i += 1;
                }
                s += 1;
            }
            video_set_color(COLOR_WHITE, COLOR_BLACK);
            video_print(b"\n  \0".as_ptr());
            video_put_dec(count as u32);
            video_print(b" \xD0\xA4\xD0\xB0\xD0\xB9\xD0\xBB(\xD0\xBE\xD0\xB2)\n\n\0".as_ptr());
            return count;
        }

        // Поддиректория
        let dn = if *path == b'/' { path.add(1) } else { path };
        let mut d83 = [b' '; 11];
        let mut di: usize = 0;
        while *dn.add(di) != 0 && *dn.add(di) != b'/' && di < 8 {
            d83[di] = toupper(*dn.add(di) as i32) as u8;
            di += 1;
        }

        let mut de: Fat16DirEntry = core::mem::zeroed();
        let mut df = 0;
        let mut s: u32 = 0;
        while s < ROOT_DIR_SECTORS && df == 0 {
            if rd(ROOT_DIR_START + s, SEC_BUF.as_mut_ptr()) != 0 {
                return -1;
            }
            for i in 0..SECTOR_SIZE / 32 {
                let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                let e_ = &*eptr.add(i);
                if e_.name[0] == 0x00 {
                    break;
                }
                if e_.attributes & 0x10 == 0 {
                    continue;
                }
                if memcmp(e_.name.as_ptr(), d83.as_ptr(), 8) == 0 {
                    de = *e_;
                    df = 1;
                    break;
                }
            }
            s += 1;
        }
        if df == 0 {
            video_print(b"  Not found\n\n\0".as_ptr());
            return -1;
        }

        let mut clus = de.cluster_low;
        while clus < 0xFFF8 {
            let lba = clus_lba(clus);
            for s in 0..BPB.sectors_per_cluster {
                if rd(lba + s as u32, SEC_BUF.as_mut_ptr()) != 0 {
                    return -1;
                }
                let mut i: usize = 0;
                while i < SECTOR_SIZE / 32 {
                    let eptr = SEC_BUF.as_ptr() as *const Fat16DirEntry;
                    let e_ = &*eptr.add(i);
                    if e_.name[0] == 0x00 {
                        clus = 0xFFFF;
                        break;
                    }
                    if e_.name[0] == 0xE5 || e_.attributes == 0x0F || e_.attributes & 0x08 != 0 {
                        i += 1;
                        continue;
                    }
                    list_common_entry(i, &mut count);
                    i += 1;
                }
            }
            clus = fat_get(clus);
        }
        video_set_color(COLOR_WHITE, COLOR_BLACK);
        video_print(b"\n  \0".as_ptr());
        video_put_dec(count as u32);
        video_print(b" \xD0\xA4\xD0\xB0\xD0\xB9\xD0\xBB(\xD0\xBE\xD0\xB2)\n\n\0".as_ptr());
        count
    }
}

pub fn fat16_list_root() -> i32 {
    fat16_list_dir(b"".as_ptr())
}

pub fn fat16_file_size(fn_: *const u8) -> u32 {
    unsafe {
        let mut e: Fat16DirEntry = core::mem::zeroed();
        if fat16_find_file(fn_, &mut e) != 0 {
            return 0;
        }
        e.file_size
    }
}

pub fn fat16_file_exists(fn_: *const u8) -> i32 {
    unsafe {
        let mut e: Fat16DirEntry = core::mem::zeroed();
        if fat16_find_file(fn_, &mut e) == 0 {
            1
        } else {
            0
        }
    }
}