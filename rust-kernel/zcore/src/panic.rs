//! Порт `kernel/panic.c` + `panic.h` — экран паники и обработчик исключений.

use core::arch::asm;
use core::ptr;

use crate::arch::cpu::{abort_user_program, disable_interrupts, system_halt};
use crate::arch::io::{inb, outb};
use crate::drivers::video::{
    video_clear, video_print, video_print_char, video_set_color, video_set_cursor, COLOR_BLACK,
    COLOR_LIGHT_RED, COLOR_WHITE,
};

const COM1: u16 = 0x3F8;

unsafe fn serial_init() {
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x80);
    outb(COM1 + 0, 0x0C);
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x03);
    outb(COM1 + 2, 0xC7);
    outb(COM1 + 4, 0x0B);
}

fn serial_putc(c: u8) {
    unsafe {
        while inb(COM1 + 5) & 0x20 == 0 {}
        outb(COM1, c);
    }
}

fn serial_puts(s: &[u8]) {
    for &c in s {
        unsafe { serial_putc(c) };
    }
}

fn serial_hex(v: u32) {
    serial_puts(b"0x");
    let mut i: i32 = 28;
    while i >= 0 {
        let n = ((v >> i) & 0xF) as u8;
        serial_putc(if n < 10 { b'0' + n } else { b'A' + n - 10 });
        i -= 4;
    }
}

#[no_mangle]
pub extern "C" fn serial_selftest() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1 + 0, 0x0C);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0xC7);
        outb(COM1 + 4, 0x0B);
    }
    let msg = b"SERIALTEST:\r\n";
    for &c in msg {
        serial_puts(&[c]);
    }
    serial_puts(b"done\r\n");
}

fn serial_dump(regs: *mut Registers, cr2: u32) {
    let r = unsafe { &*regs };
    unsafe { serial_init() };
    serial_puts(b"\r\n[EXC] int=");
    serial_hex(r.int_num);
    serial_puts(b" err=");
    serial_hex(r.err_code);
    serial_puts(b" cr2=");
    serial_hex(cr2);
    serial_puts(b" eip=");
    serial_hex(r.eip);
    serial_puts(b" cs=");
    serial_hex(r.cs);
    serial_puts(b" efl=");
    serial_hex(r.eflags);
    serial_puts(b" esp=");
    serial_hex(r.esp);
    serial_puts(b" ss=");
    serial_hex(r.ss);
    serial_puts(b" eax=");
    serial_hex(r.eax);
    serial_puts(b" ebx=");
    serial_hex(r.ebx);
    serial_puts(b" ecx=");
    serial_hex(r.ecx);
    serial_puts(b" edx=");
    serial_hex(r.edx);
    serial_puts(b" esi=");
    serial_hex(r.esi);
    serial_puts(b" edi=");
    serial_hex(r.edi);
    serial_puts(b" ebp=");
    serial_hex(r.ebp);
    serial_puts(b"\r\n");
}

/// Фрейм регистров, сохранённых обработчиком исключения (`registers_t`).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct Registers {
    pub gs: u32,
    pub fs: u32,
    pub es: u32,
    pub ds: u32,
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp_dummy: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub int_num: u32,
    pub err_code: u32,
    pub eip: u32,
    pub cs: u32,
    pub eflags: u32,
    pub esp: u32,
    pub ss: u32,
}

static EXCEPTION_NAMES: [&[u8]; 20] = [
    b"Division By Zero\0",
    b"Debug\0",
    b"Non Maskable Interrupt\0",
    b"Breakpoint\0",
    b"Overflow\0",
    b"Bound Range Exceeded\0",
    b"Invalid Opcode\0",
    b"Device Not Available\0",
    b"Double Fault\0",
    b"Coprocessor Segment\0",
    b"Invalid TSS\0",
    b"Segment Not Present\0",
    b"Stack-Segment Fault\0",
    b"General Protection Fault\0",
    b"Page Fault\0",
    b"Reserved\0",
    b"x87 FPU Error\0",
    b"Alignment Check\0",
    b"Machine Check\0",
    b"SIMD FP Exception\0",
];

const PANIC_COLOR_BG: u32 = (15 << 16) | (24 << 8) | 48; // Тёмно-синий
const PANIC_COLOR_TEXT: u32 = (180 << 16) | (200 << 8) | 255; // Светло-голубой
const PANIC_COLOR_WHITE: u32 = 0xFFFFFF;
const PANIC_COLOR_REG_KEY: u32 = (255 << 16) | (220 << 8) | 50; // Жёлтый

fn put_hex(v: u32) {
    const HEX: *const u8 = b"0123456789ABCDEF".as_ptr();
    video_print(b"0x\0".as_ptr());
    let mut i: i32 = 28;
    while i >= 0 {
        unsafe { video_print_char(*HEX.add(((v >> i) & 0xF) as usize)); }
        i -= 4;
    }
}

fn video_print_bar() {
    for _ in 0..100 {
        video_print_char(b' ');
    }
}

fn panic_screen(title: *const u8, msg: *const u8, regs: Option<*const Registers>) {
    unsafe { disable_interrupts() };

    if let Some(rp) = regs {
        serial_dump(rp as *mut Registers, 0);
    } else {
        serial_puts(b"\r\n[PANIC] no-regs\r\n");
    }

    video_clear(PANIC_COLOR_BG);

    video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
    video_print_bar();
    video_set_cursor(0, 1);
    video_print_bar();
    video_set_cursor(2, 1);
    video_print(b"!!!KERNEL PANIC!!!\0".as_ptr());

    video_set_color(PANIC_COLOR_TEXT, PANIC_COLOR_BG);

    video_set_cursor(0, 3);
    video_print(b"  Exception: \0".as_ptr());
    video_print(title);

    if !msg.is_null() {
        video_set_cursor(0, 4);
        video_print(b"  \0".as_ptr());
        video_print(msg);
    }

    if regs.is_some() {
        let r = unsafe { &**regs.as_ref().unwrap() };
        video_set_color(PANIC_COLOR_TEXT, PANIC_COLOR_BG);
        video_set_cursor(0, 6);
        video_print(b"  Error code: \0".as_ptr());
        put_hex(r.err_code);
        video_print(b"    EIP: \0".as_ptr());
        put_hex(r.eip);

        video_set_cursor(2, 8);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"EAX=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.eax);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"  EBX=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.ebx);

        video_set_cursor(2, 9);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"ECX=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.ecx);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"  EDX=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.edx);

        video_set_cursor(2, 10);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"ESI=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.esi);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"  EDI=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.edi);

        video_set_cursor(2, 11);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"EBP=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.ebp);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"  ESP=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.esp);

        video_set_cursor(2, 12);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"EIP=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.eip);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"  EFL=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.eflags);

        video_set_cursor(2, 13);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b" CS=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.cs);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"   SS=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.ss);

        video_set_cursor(2, 14);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b" DS=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.ds);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(b"   ES=\0".as_ptr());
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(r.es);
    }

    video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
    video_set_cursor(0, 17);
    video_print(b"The system has stopped working. Restart your computer.\n\n\0".as_ptr());

    video_print(
        b"            %%%%%                \n          %*     %               \n          %    -...%%\n          %#  %..%:...*%*\n           %   %......%+..%      \n           %%   .%.......%%      \n         %%  %-   %%....%        \n       #%     %     %            \n      %*      %      %           \n      % %    %       %           \n     % % %%%  +%%    %#%*        \n     % %%    .....% %:..%        \n        %%:  %....*%...%         \n           *%%%%%#  %%           ".as_ptr(),
    );

    system_halt();
}

/// Обработчик исключений (вызывается из `isr_common` в entry.S).
#[no_mangle]
pub extern "C" fn exception_handler(regs: *mut Registers) {
    let r = unsafe { &*regs };
    unsafe { serial_init() };
    serial_puts(b"\r\n[EH] enter\r\n");
    let name: *const u8 = if r.int_num < 20 {
        EXCEPTION_NAMES[r.int_num as usize].as_ptr()
    } else {
        b"Unknown Exception".as_ptr()
    };
    serial_dump(regs, 0);

    // Если это исключение пришло из Ring2 - не вызывать панику.
    // Сделать возврат программы будто она завершилась с ошибкой.
    if (r.cs & 0x3) == 2 {
        video_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
        video_print(b"\n[PROGRAM FAULT] \0".as_ptr());
        video_set_color(COLOR_WHITE, COLOR_BLACK);
        video_print(name);

        if r.int_num == 14 {
            let cr2: u32;
            unsafe {
                asm!("mov {0}, cr2", out(reg) cr2);
            }
            video_print(b" at \0".as_ptr());
            put_hex(cr2);
            video_print(b" err=\0".as_ptr());
            put_hex(r.err_code);
        }
        video_print(b"\n\0".as_ptr());

        // 0xE0000000 | int_num - чтобы видеть причину в ZXE-загрузчике
        unsafe { abort_user_program(0xE0000000u32 | (r.int_num & 0xFFu32)) };
        return;
    }

    // Kernel fault
    if r.int_num == 14 {
        let cr2: u32;
        unsafe {
            asm!("mov {0}, cr2", out(reg) cr2);
        }
        video_print(b"\n[PF] CR2=\0".as_ptr());
        put_hex(cr2);
        video_print(b" ERR=\0".as_ptr());
        put_hex(r.err_code);
        video_print(b"\n\0".as_ptr());
    }
    panic_screen(name, ptr::null(), Some(regs));
}

/// Ручной вызов (порт `kernel_panic`).
pub fn kernel_panic(msg: *const u8) {
    panic_screen(b"Kernel Panic".as_ptr(), msg, None);
}