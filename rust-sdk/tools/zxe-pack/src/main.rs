//! zxe-pack — упаковщик ELF-файла программы в формат .ZXE для Zinux.
//!
//! Использование: zxe-pack <input.elf> <output.zxe> [name]
//!
//! Разбирает ELF32, забирает PT_LOAD-сегменты в пользовательской памяти
//! (0x200000..0x300000), сшивает их в плоский образ (с нулевым заполнением
//! промежутков выравнивания) и дописывает 48-байтный заголовок ZXE:
//!
//! | смещение | поле           | описание                       |
//! |----------|----------------|--------------------------------|
//! | 0x00     | magic  u32     | 0x01455A58 ("ZXE\x01")         |
//! | 0x04     | version u8     | 1                              |
//! | 0x05     | flags u8       | 0                              |
//! | 0x06     | header_size u16| 48                             |
//! | 0x08     | code_size u32  | размер машинного кода          |
//! | 0x0C     | data_size u32  | 0 (данные уже входят в код)    |
//! | 0x10     | bss_size u32   | размер неинициализированных данных |
//! | 0x14     | entry_offset u32 | смещение точки входа от кода  |
//! | 0x18     | name[16]       | имя программы (NUL-terminated) |
//! | 0x28     | reserved[8]    | зарезервировано                |

use std::env;
use std::fs;
use std::process;

const ZXE_MAGIC: u32 = 0x0145_5A58;
const ZXE_VERSION: u8 = 1;
const ZXE_HEADER_SIZE: u16 = 48;
const ZXE_LOAD_ADDRESS: u32 = 0x0020_0000;
const USER_MEM_START: u32 = 0x0020_0000;
const USER_MEM_END: u32 = 0x0030_0000;
const USER_STACK_TOP: u32 = 0x002F_0000;
const SOFT_LIMIT: u64 = 64 * 1024;
const HARD_LIMIT: u64 = 2048 * 1024;

const PT_LOAD: u32 = 1;

#[derive(Clone, Copy)]
struct Segment {
    vaddr: u32,
    offset: u32,
    filesz: u32,
    memsz: u32,
}

fn parse_elf(data: &[u8]) -> (u32, Vec<Segment>) {
    assert!(data.len() >= 52, "too small to be an ELF32");
    assert!(&data[0..4] == b"\x7fELF", "not an ELF file");
    assert_eq!(data[4], 1, "not a 32-bit ELF");
    assert!((data[5] as usize & 2) == 0, "not little-endian");

    let e_entry = u32::from_le_bytes(data[24..28].try_into().unwrap());
    let e_phoff = u32::from_le_bytes(data[28..32].try_into().unwrap()) as usize;
    let e_phentsize = u16::from_le_bytes(data[42..44].try_into().unwrap()) as usize;
    let e_phnum = u16::from_le_bytes(data[44..46].try_into().unwrap()) as usize;

    assert!(e_phentsize >= 32, "strange phentsize");
    let table_end = e_phoff + e_phnum * e_phentsize;
    assert!(table_end <= data.len(), "program header table out of range");

    let mut segs = Vec::new();
    for i in 0..e_phnum {
        let off = e_phoff + i * e_phentsize;
        let p_type = u32::from_le_bytes(data[off..off + 4].try_into().unwrap());
        let p_offset = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap());
        let p_vaddr = u32::from_le_bytes(data[off + 8..off + 12].try_into().unwrap());
        let p_filesz = u32::from_le_bytes(data[off + 16..off + 20].try_into().unwrap());
        let p_memsz = u32::from_le_bytes(data[off + 20..off + 24].try_into().unwrap());
        if p_type != PT_LOAD {
            continue;
        }
        let seg = Segment { vaddr: p_vaddr, offset: p_offset, filesz: p_filesz, memsz: p_memsz };
        if seg.filesz > 0 && seg.vaddr >= USER_MEM_START && seg.vaddr < USER_MEM_END {
            let end = seg.vaddr as u64 + seg.filesz as u64;
            assert!(end <= USER_MEM_END as u64, "segment exceeds user memory");
            segs.push(seg);
        }
    }
    assert!(!segs.is_empty(), "no loadable segments in user memory 0x200000..0x300000");
    segs.sort_by_key(|s| s.vaddr);
    (e_entry, segs)
}

fn build_image(elf: &[u8], segs: &[Segment]) -> (Vec<u8>, u32, u64) {
    let mut image = Vec::new();
    let first_vaddr = segs[0].vaddr;
    let mut bss: u64 = 0;

    if first_vaddr > ZXE_LOAD_ADDRESS + ZXE_HEADER_SIZE as u32 {
        let pad = (first_vaddr - ZXE_LOAD_ADDRESS - ZXE_HEADER_SIZE as u32) as usize;
        image.resize(pad, 0);
    }

    for s in segs {
        let need = (s.vaddr as u64 - first_vaddr as u64) as usize;
        if image.len() < need {
            image.resize(need, 0);
        } else if image.len() > need {
            panic!("segments overlap in image");
        }
        let start = s.offset as usize;
        let end = start + s.filesz as usize;
        assert!(end <= elf.len(), "segment data out of range");
        image.extend_from_slice(&elf[start..end]);
        bss += (s.memsz as u64).saturating_sub(s.filesz as u64);
    }

    let top = first_vaddr as u64 + image.len() as u64;
    assert!(top <= USER_MEM_END as u64, "image end beyond user memory");
    if top > USER_STACK_TOP as u64 {
        eprintln!("warning: image reaches into stack area (0x2F0000)");
    }
    if image.len() as u64 + 48 > SOFT_LIMIT {
        eprintln!(
            "warning: image {:.1} KiB + header exceeds the 64 KiB soft limit",
            (image.len() as f64 + 48.0) / 1024.0
        );
    }
    assert!(image.len() as u64 + 48 <= HARD_LIMIT, "image exceeds 2 MiB hard limit");

    let entry_offset = first_vaddr.saturating_sub(ZXE_LOAD_ADDRESS + ZXE_HEADER_SIZE as u32);
    (image, bss as u32, entry_offset as u64)
}

fn make_header(code_size: u32, bss_size: u32, entry_offset: u32, name: &str) -> [u8; 48] {
    let mut h = [0u8; 48];
    h[0..4].copy_from_slice(&ZXE_MAGIC.to_le_bytes());
    h[4] = ZXE_VERSION;
    h[5] = 0;
    h[6..8].copy_from_slice(&ZXE_HEADER_SIZE.to_le_bytes());
    h[8..12].copy_from_slice(&code_size.to_le_bytes());
    h[12..16].copy_from_slice(&0u32.to_le_bytes()); // data_size
    h[16..20].copy_from_slice(&bss_size.to_le_bytes());
    h[20..24].copy_from_slice(&entry_offset.to_le_bytes());
    let name_bytes = name.as_bytes();
    let n = name_bytes.len().min(15);
    h[24..24 + n].copy_from_slice(&name_bytes[..n]);
    h // reserved[8] остаётся нулевым
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input.elf> <output.zxe> [name]", args[0]);
        process::exit(1);
    }
    let input = &args[1];
    let output = &args[2];
    let name = args.get(3).map(|s| s.as_str()).unwrap_or("rust");

    let elf = fs::read(input).unwrap_or_else(|e| {
        eprintln!("error: cannot read {}: {}", input, e);
        process::exit(1)
    });

    let (entry, segs) = parse_elf(&elf);
    let first_vaddr = segs[0].vaddr;
    let (image, bss_size, entry_offset_from_base) = build_image(&elf, &segs);

    let entry = if entry >= first_vaddr { entry - first_vaddr } else { 0 };
    let header = make_header(image.len() as u32, bss_size, entry, name);

    fs::write(output, [&header[..], &image[..]].concat()).unwrap_or_else(|e| {
        eprintln!("error: cannot write {}: {}", output, e);
        process::exit(1)
    });

    println!("{} -> {}:", input, output);
    println!("  code    : {} bytes ({} B + padding)", image.len(), image.len());
    println!("  bss     : {} bytes", bss_size);
    println!("  entry   : 0x{:08X} (offset {} from 0x200030)", first_vaddr + entry, entry_offset_from_base + entry as u64);
    println!("  total   : {} bytes (header {} + code {})", 48 + image.len(), 48, image.len());
    println!("  size    : {:.1} KiB (limit: 64 KiB)", (48 + image.len()) as f64 / 1024.0);
}