//! Порт `kernel/system.c` (TSS-часть): задача переключения на ring0.

use core::arch::asm;

/// Дескриптор TSS в GDT (`gdt_entry_t`).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct GdtEntry {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_mid: u8,
    pub access: u8,
    pub gran: u8,
    pub base_high: u8,
}

/// TSS (совпадает с `tss_entry_t`, packed).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct TssEntry {
    pub prev_tss: u16,
    pub reserved0: u16,
    pub esp0: u32,
    pub ss0: u16,
    pub reserved1: u16,
    pub esp1: u32,
    pub ss1: u16,
    pub reserved2: u16,
    pub esp2: u32,
    pub ss2: u16,
    pub reserved3: u16,
    pub cr3: u32,
    pub eip: u32,
    pub eflags: u32,
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,
    pub es: u16,
    pub reserved4: u16,
    pub cs: u16,
    pub reserved5: u16,
    pub ss: u16,
    pub reserved6: u16,
    pub ds: u16,
    pub reserved7: u16,
    pub fs: u16,
    pub reserved8: u16,
    pub gs: u16,
    pub reserved9: u16,
    pub ldt: u16,
    pub reserved10: u16,
    pub trap: u16,
    pub iomap_base: u16,
}

/// Индекс TSS-дескриптора в GDT (0x28 = 5-й дескриптор).
pub const TSS_SELECTOR: u16 = 0x28;

#[repr(C, packed)]
struct Gdtr {
    limit: u16,
    base: u32,
}

static mut KERNEL_TSS: TssEntry = TssEntry {
    prev_tss: 0, reserved0: 0, esp0: 0, ss0: 0, reserved1: 0,
    esp1: 0, ss1: 0, reserved2: 0, esp2: 0, ss2: 0, reserved3: 0,
    cr3: 0, eip: 0, eflags: 0,
    eax: 0, ecx: 0, edx: 0, ebx: 0,
    esp: 0, ebp: 0, esi: 0, edi: 0,
    es: 0, reserved4: 0, cs: 0, reserved5: 0,
    ss: 0, reserved6: 0, ds: 0, reserved7: 0,
    fs: 0, reserved8: 0, gs: 0, reserved9: 0,
    ldt: 0, reserved10: 0, trap: 0, iomap_base: 0,
};

/// Установка дескриптора TSS в GDT (порт `gdt_set_tss_descriptor`).
fn gdt_set_tss_descriptor(gdt: *mut GdtEntry, index: usize, base: u32, limit: u32) {
    unsafe {
        (*gdt.add(index)).limit_low = limit as u16;
        (*gdt.add(index)).base_low = base as u16;
        (*gdt.add(index)).base_mid = ((base >> 16) & 0xFF) as u8;
        (*gdt.add(index)).access = 0x89; // present, ring0, available 32-bit TSS
        (*gdt.add(index)).gran = ((limit >> 16) & 0x0F) as u8;
        (*gdt.add(index)).base_high = ((base >> 24) & 0xFF) as u8;
    }
}

/// Инициализация TSS (порт `tss_init`).
pub fn tss_init() {
    let mut gdtr = Gdtr { limit: 0, base: 0 };
    unsafe {
        asm!(
            "sgdt [{0}]",
            in(reg) core::ptr::addr_of_mut!(gdtr),
            options(nostack)
        );
    }

    unsafe {
        let tss = &mut *core::ptr::addr_of_mut!(KERNEL_TSS);
        // memset, как в C
        core::ptr::write_bytes(tss as *mut TssEntry as *mut u8, 0, core::mem::size_of::<TssEntry>());
        tss.ss0 = 0x10;
        tss.esp0 = 0x9D000;
        tss.iomap_base = core::mem::size_of::<TssEntry>() as u16;

        let gdt = gdtr.base as *mut GdtEntry;
        gdt_set_tss_descriptor(gdt, 5, core::ptr::addr_of_mut!(KERNEL_TSS) as u32,
                               core::mem::size_of::<TssEntry>() as u32 - 1);
    }

    unsafe {
        asm!(
            "ltr {0:x}",
            in(reg) TSS_SELECTOR as u32,
            options(nostack)
        );
    }
}