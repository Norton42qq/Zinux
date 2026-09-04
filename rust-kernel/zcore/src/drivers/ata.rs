//! Порт `kernel/bios_disk.c` + `drivers/ata.h` — диск: RAM-буфер stage2 или ATA PIO.

use zlibk::mem::{memcpy, memset};

use crate::arch::io::{inb, inw, outb, outw};

// Адреса для stage2
const DISK_BUF_FLAG: u32 = 0x8014;
const DISK_BUF_DRIVE: u32 = 0x8018;
const DISK_BUF_BASE: *mut u8 = 0x50000 as *mut u8;
const SECTOR_SIZE: usize = 512;
const FAT_PART_START: u32 = 2048;
const MAX_DIRTY: u32 = 1024;

static mut DIRTY_BITMAP: [u8; MAX_DIRTY as usize / 8 + 1] = [0; MAX_DIRTY as usize / 8 + 1];

fn dirty_set(i: u32) {
    unsafe {
        if i < MAX_DIRTY {
            DIRTY_BITMAP[(i / 8) as usize] |= 1u8 << (i % 8);
        }
    }
}

fn dirty_clr(i: u32) {
    unsafe {
        if i < MAX_DIRTY {
            DIRTY_BITMAP[(i / 8) as usize] &= !(1u8 << (i % 8));
        }
    }
}

fn dirty_get(i: u32) -> i32 {
    unsafe {
        if i >= MAX_DIRTY {
            return 0;
        }
        ((DIRTY_BITMAP[(i / 8) as usize] >> (i % 8)) & 1) as i32
    }
}

// Трамплин PM->RM для записи на накопитель (в entry.S).
extern "C" {
    fn pm_to_rm_write(lba: u32, drive: u8, buf: *mut u8) -> i32;
}

// HW ATA PIO
const ATA_DATA: u16 = 0x1F0;
const ATA_STATUS: u16 = 0x1F7;
const ATA_COMMAND: u16 = 0x1F7;
const ATA_DRIVE: u16 = 0x1F6;
const ATA_SECTOR_CNT: u16 = 0x1F2;
const ATA_LBA_LO: u16 = 0x1F3;
const ATA_LBA_MID: u16 = 0x1F4;
const ATA_LBA_HI: u16 = 0x1F5;
const ATA_SR_BSY: u8 = 0x80;
const ATA_SR_DRDY: u8 = 0x40;
const ATA_SR_DRQ: u8 = 0x08;
const ATA_SR_ERR: u8 = 0x01;
const ATA_CMD_READ: u8 = 0x20;
const ATA_CMD_WRITE: u8 = 0x30;
const ATA_CMD_FLUSH: u8 = 0xE7;

static mut BUF_SECTORS: u32 = 0;
static mut BUF_VALID: i32 = 0;
static mut ATA_HW_OK: i32 = 0;
static mut BOOT_DRIVE: u8 = 0x80;

fn hw_wait_ready() -> i32 {
    for _ in 0..100000i32 {
        let s = unsafe { inb(ATA_STATUS) };
        if s == 0xFF {
            return -1;
        }
        if s & ATA_SR_BSY == 0 && s & ATA_SR_DRDY != 0 {
            return 0;
        }
    }
    -1
}

fn hw_wait_drq() -> i32 {
    for _ in 0..100000i32 {
        let s = unsafe { inb(ATA_STATUS) };
        if s == 0xFF {
            return -1;
        }
        if s & ATA_SR_ERR != 0 {
            return -1;
        }
        if s & ATA_SR_DRQ != 0 {
            return 0;
        }
    }
    -1
}

fn hw_read(lba: u32, buf: *mut u8) -> i32 {
    let b = buf as *mut u16;
    if hw_wait_ready() != 0 {
        return -1;
    }
    unsafe {
        outb(ATA_DRIVE, 0xE0u8 | ((lba >> 24) & 0x0F) as u8);
        outb(ATA_SECTOR_CNT, 1);
        outb(ATA_LBA_LO, lba as u8);
        outb(ATA_LBA_MID, (lba >> 8) as u8);
        outb(ATA_LBA_HI, (lba >> 16) as u8);
        outb(ATA_COMMAND, ATA_CMD_READ);
    }
    if hw_wait_drq() != 0 {
        return -1;
    }
    unsafe {
        for i in 0..256usize {
            b.add(i).write_volatile(inw(ATA_DATA));
        }
    }
    0
}

fn hw_write(lba: u32, buf: *const u8) -> i32 {
    let b = buf as *const u16;
    if hw_wait_ready() != 0 {
        return -1;
    }
    unsafe {
        outb(ATA_DRIVE, 0xE0u8 | ((lba >> 24) & 0x0F) as u8);
        outb(ATA_SECTOR_CNT, 1);
        outb(ATA_LBA_LO, lba as u8);
        outb(ATA_LBA_MID, (lba >> 8) as u8);
        outb(ATA_LBA_HI, (lba >> 16) as u8);
        outb(ATA_COMMAND, ATA_CMD_WRITE);
    }
    if hw_wait_drq() != 0 {
        return -1;
    }
    unsafe {
        for i in 0..256usize {
            outw(ATA_DATA, b.add(i).read_volatile());
        }
        outb(ATA_COMMAND, ATA_CMD_FLUSH);
    }
    let _ = hw_wait_ready();
    0
}

// Публичный API (порт `drivers/ata.h`)

pub fn ata_init() {
    unsafe {
        memset(DIRTY_BITMAP.as_mut_ptr(), 0, DIRTY_BITMAP.len());

        BUF_SECTORS = read_volatile_u32(DISK_BUF_FLAG);
        BUF_VALID = if BUF_SECTORS > 0 { 1 } else { 0 };
        BOOT_DRIVE = read_volatile_u8(DISK_BUF_DRIVE);

        if inb(ATA_STATUS) == 0xFF {
            ATA_HW_OK = 0;
            return;
        }
        outb(ATA_DRIVE, 0xA0);
        for _ in 0..15i32 {
            let _ = inb(ATA_STATUS);
        }
        if hw_wait_ready() == 0 {
            ATA_HW_OK = 1;
        }
    }
}

pub fn ata_read_sector(lba: u32, buffer: *mut u8) -> i32 {
    unsafe {
        if BUF_VALID != 0 {
            if lba < FAT_PART_START {
                return -1;
            }
            let idx = lba - FAT_PART_START;
            if idx >= BUF_SECTORS {
                return -1;
            }
            memcpy(buffer, DISK_BUF_BASE.add((idx as usize) * SECTOR_SIZE), SECTOR_SIZE);
            return 0;
        }
        if ATA_HW_OK != 0 {
            return hw_read(lba, buffer);
        }
    }
    -1
}

pub fn ata_write_sector(lba: u32, buffer: *const u8) -> i32 {
    unsafe {
        if BUF_VALID != 0 {
            if lba < FAT_PART_START {
                return -1;
            }
            let idx = lba - FAT_PART_START;
            if idx >= BUF_SECTORS {
                return -1;
            }
            memcpy(DISK_BUF_BASE.add((idx as usize) * SECTOR_SIZE), buffer, SECTOR_SIZE);
            dirty_set(idx);
            return 0;
        }
        if ATA_HW_OK != 0 {
            return hw_write(lba, buffer);
        }
    }
    -1
}

pub fn ata_read_sectors(lba: u32, count: u32, buffer: *mut u8) -> i32 {
    for i in 0..count {
        unsafe {
            if ata_read_sector(lba + i, buffer.add((i as usize) * SECTOR_SIZE)) != 0 {
                return -1;
            }
        }
    }
    0
}

pub fn ata_write_sectors(lba: u32, count: u32, buffer: *const u8) -> i32 {
    for i in 0..count {
        unsafe {
            if ata_write_sector(lba + i, buffer.add((i as usize) * SECTOR_SIZE)) != 0 {
                return -1;
            }
        }
    }
    0
}

pub fn ata_flush() -> i32 {
    unsafe {
        if BUF_VALID == 0 {
            return 0;
        }

        let mut errors = 0;
        let limit = if BUF_SECTORS < MAX_DIRTY { BUF_SECTORS } else { MAX_DIRTY };

        for i in 0..limit {
            if dirty_get(i) == 0 {
                continue;
            }

            let lba = FAT_PART_START + i;
            let sect = DISK_BUF_BASE.add((i as usize) * SECTOR_SIZE);
            let mut ok = -1;

            if ATA_HW_OK != 0 {
                ok = hw_write(lba, sect);
            } else {
                ok = pm_to_rm_write(lba, BOOT_DRIVE, sect);
            }

            if ok == 0 {
                dirty_clr(i);
            } else {
                errors += 1;
            }
        }
        -errors
    }
}

pub fn ata_dirty_count() -> i32 {
    unsafe {
        let mut count = 0;
        let limit = if BUF_SECTORS < MAX_DIRTY { BUF_SECTORS } else { MAX_DIRTY };
        for i in 0..limit {
            if dirty_get(i) != 0 {
                count += 1;
            }
        }
        count
    }
}

pub fn ata_identify() -> i32 {
    unsafe { if BUF_VALID != 0 || ATA_HW_OK != 0 { 0 } else { -1 } }
}

fn read_volatile_u32(addr: u32) -> u32 {
    unsafe { (addr as *const u32).read_volatile() }
}

fn read_volatile_u8(addr: u32) -> u8 {
    unsafe { (addr as *const u8).read_volatile() }
}