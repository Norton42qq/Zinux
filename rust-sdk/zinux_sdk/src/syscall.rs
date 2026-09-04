//! Низкоуровневые обёртки над системным прерыванием `int 0x80` (Zinux, Ring 2).
//!
//! Прямой вызов функций ядра по адресам запрещён (#GP), все обращения к ядру
//! выполняются только через trap gate INT 0x80 (DPL=3). Ядро (`syscall_handler`
//! в kernel/kernel_entry.asm) восстанавливает после себя все регистры, кроме
//! EAX — туда кладётся возвращаемое значение:
//!
//! | EAX | вызов            | аргументы              | возврат |
//! |-----|------------------|------------------------|---------|
//! | 1   | putc             | символ в BL/EBX        | —       |
//! | 2   | exit             | код выхода в EBX       | не вернётся |
//! | 3   | api              | EBX = ID, ECX = args   | EAX     |

/// Вывод одного символа через `int 0x80` (EAX=1, символ в BL).
#[inline(always)]
pub fn syscall_putc(c: u8) {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("eax") 1u32,
            in("ebx") c as u32,
            options(nostack, nomem)
        );
    }
}

/// Завершение программы (EAX=2, код выхода в EBX). Никогда не возвращается.
#[inline(never)]
pub fn syscall_exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("eax") 2u32,
            in("ebx") code as u32,
            options(noreturn, nostack)
        );
    }
}

/// Вызов API-функции ядра (EAX=3, EBX = ID функции, ECX = указатель на массив
/// аргументов `u32`). Ядро читает до 8 dword из массива и возвращает результат
/// в EAX. Указатель `arg` должен быть в пользовательской памяти
/// (0x200000..0x300000).
#[inline(always)]
pub fn syscall_api(sys_id: u32, arg: *const ()) -> u32 {
    let ret: u32;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("eax") 3u32 => ret,
            in("ebx") sys_id,
            in("ecx") arg,
            lateout("edx") _,
            lateout("edi") _,
        );
    }
    ret
}

/// Удобная обёртка: строит массив аргументов на стеке и вызывает syscall_api.
#[inline(always)]
pub fn api_call(sys_id: u32, args: &[u32]) -> u32 {
    let mut buf = [0u32; 8];
    let n = if args.len() > 8 { 8 } else { args.len() };
    buf[..n].copy_from_slice(&args[..n]);
    syscall_api(sys_id, buf.as_ptr() as *const ())
}
