//! Порт `kernel/system.c` (CPU-часть): системная информация, лёгкие asm-хелперы.

use core::arch::asm;

use super::io::{inb, outb};

/// Системная информация (из `system.h`).
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SystemInfo {
    pub mem_lower: u32,       // Память ниже 1MB (KB)
    pub mem_upper: u32,       // Память выше 1MB (KB)
    pub cpu_vendor: [u32; 4], // Строка производителя CPU
    pub cpu_features: u32,    // Флаги функций CPU
}

static mut SYS_INFO: SystemInfo = SystemInfo {
    mem_lower: 0,
    mem_upper: 0,
    cpu_vendor: [0; 4],
    cpu_features: 0,
};

/// Функции из `arch/entry.S`.
extern "C" {
    pub fn enable_interrupts();
    pub fn disable_interrupts();
    pub fn run_user_program(entry: u32, user_stack_top: u32, argc: i32, argv: *mut *mut u8) -> i32;
    pub fn abort_user_program(exit_code: u32);
}

/// Обнаружение процессора (порт `detect_cpu`).
fn detect_cpu() {
    // Проверка поддержки CPUID (pushfl-трюк)
    let eax: u32;
    unsafe {
        asm!(
            "pushfd",
            "pushfd",
            "xor dword ptr [esp], 0x200000",
            "popfd",
            "pushfd",
            "pop {0:e}",
            "xor {0:e}, dword ptr [esp]",
            "popfd",
            "and {0:e}, 0x200000",
            out(reg) eax,
            options(nostack)
        );
    }

    if eax == 0 {
        // CPUID не поддерживается
        unsafe {
            SYS_INFO.cpu_vendor = [0x6E6B6E55, 0x6E776F6E, 0, 0];
            SYS_INFO.cpu_features = 0;
        }
        return;
    }

    // Строка производителя (CPUID.0)
    let (mut _eax, mut ebx, mut ecx, mut edx): (u32, u32, u32, u32);
    unsafe {
        asm!(
            "cpuid",
            inout("eax") 0u32 => _eax,
            out("ebx") ebx,
            out("ecx") ecx,
            out("edx") edx,
            options(nostack)
        );
        SYS_INFO.cpu_vendor = [ebx, edx, ecx, 0];
    }

    // Флаги функций (CPUID.1)
    unsafe {
        asm!(
            "cpuid",
            inout("eax") 1u32 => _eax,
            out("ebx") _eax,
            out("ecx") ecx,
            out("edx") edx,
            options(nostack)
        );
        SYS_INFO.cpu_features = edx;
    }
}

/// Определение размера памяти (порт `detect_memory`): CMOS 17h/18h.
fn detect_memory() {
    unsafe {
        SYS_INFO.mem_lower = 640;

        outb(0x70, 0x17);
        let low = inb(0x71);
        outb(0x70, 0x18);
        let high = inb(0x71);

        SYS_INFO.mem_upper = ((high as u32) << 8) | low as u32;
        if SYS_INFO.mem_upper == 0 {
            SYS_INFO.mem_upper = 15 * 1024;
        }
    }
}

/// Инициализация системы (порт `system_init`).
pub fn system_init() {
    detect_cpu();
    detect_memory();
    super::paging::paging_init();
    super::tss::tss_init();
    super::idt::idt_init();
}

/// Получение системной информации (порт `system_get_info`).
pub fn system_get_info() -> *mut SystemInfo {
    core::ptr::addr_of_mut!(SYS_INFO)
}

/// Остановка системы (порт `system_halt`).
pub fn system_halt() -> ! {
    unsafe { disable_interrupts() };
    loop {
        unsafe { asm!("hlt", options(nostack)); }
    }
}

/// Перезагрузка системы (порт `system_reboot`).
pub fn system_reboot() -> ! {
    unsafe { disable_interrupts() };

    // Метод 1: через клавиатурный контроллер
    let mut good = 0x02u8;
    while good & 0x02 != 0 {
        unsafe { good = inb(0x64); }
    }
    unsafe { outb(0x64, 0xFE); }

    // Если не сработало — halt
    system_halt()
}