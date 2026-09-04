//! Перенос `kernel/system.c` (syscall-часть) + `kernel/kernel.c` (`kernel_main`).
//!
//! Системные вызовы INT 0x80:
//!   EAX=1 -> `syscall_putc` (EBX = символ)
//!   EAX=2 -> выход из пользовательской программы (обрабатывается в entry.S)
//!   EAX=3 -> `syscall_api_call` (EBX = id, ECX = args)

use core::ptr;

use zlibk::alloc::memory_init;

use crate::arch::cpu::{enable_interrupts, system_halt, system_init};
use crate::arch::paging::{USER_MEM_END, USER_MEM_START};
use crate::arch::pit::{pit_init, PIT_TARGET_HZ};
use crate::panic::serial_selftest;
use crate::drivers::ata::{ata_dirty_count, ata_flush, ata_init};
use crate::drivers::keyboard::keyboard_init;
use crate::drivers::mouse::{mouse_init, mouse_is_ready};
use crate::drivers::video as v;
use crate::fat16::fat16_init;
use crate::shell::{shell_init, shell_load_config, shell_run, shell_set_fs_ready};

// --------------------------------------------------------------------------
// Проверка пользовательских указателей
// --------------------------------------------------------------------------

/// Проверка, что [p, p+size) лежит в пользовательской памяти (`user_ptr_ok`).
fn user_ptr_ok(p: *const u8, size: u32) -> i32 {
    if p.is_null() {
        return 0;
    }
    let a = p as u32;
    if a < USER_MEM_START || a >= USER_MEM_END {
        return 0;
    }
    if size == 0 {
        return 1;
    }
    if size > USER_MEM_END - a {
        return 0;
    }
    1
}

/// Проверка строки с ограничением длины (`user_str_ok`).
fn user_str_ok(s: *const u8, max_len: u32) -> i32 {
    if s.is_null() || max_len == 0 {
        return 0;
    }
    let a = s as u32;
    if a < USER_MEM_START || a >= USER_MEM_END {
        return 0;
    }
    let mut i = 0u32;
    while i < max_len && (a + i) < USER_MEM_END {
        if unsafe { *s.add(i as usize) } == 0 {
            return 1;
        }
        i += 1;
    }
    0
}

// --------------------------------------------------------------------------
// Системные вызовы
// --------------------------------------------------------------------------

/// Синхронный вывод в консоль (INT 0x80, EAX=1).
#[no_mangle]
pub extern "C" fn syscall_putc(ch: u32) {
    v::video_print_char((ch & 0xFF) as u8);
}

// ID функций API (syscall_api_ids.h, 1..65).
const SYSCALL_API_PRINT: u32 = 1;
const SYSCALL_API_PRINTLN: u32 = 2;
const SYSCALL_API_PRINT_CHAR: u32 = 3;
const SYSCALL_API_PRINT_INT: u32 = 4;
const SYSCALL_API_PRINT_HEX: u32 = 5;
const SYSCALL_API_SET_COLOR: u32 = 6;
const SYSCALL_API_READ_CHAR: u32 = 7;
const SYSCALL_API_READ_LINE: u32 = 8;
const SYSCALL_API_CLEAR_SCREEN: u32 = 9;
const SYSCALL_API_SET_CURSOR: u32 = 10;
const SYSCALL_API_GET_CURSOR: u32 = 11;
const SYSCALL_API_DRAW_BOX: u32 = 12;
const SYSCALL_API_DRAW_BOX_TITLED: u32 = 13;
const SYSCALL_API_FILL_RECT: u32 = 14;
const SYSCALL_API_DRAW_HLINE: u32 = 15;
const SYSCALL_API_DRAW_VLINE: u32 = 16;
const SYSCALL_API_DRAW_WINDOW: u32 = 17;
const SYSCALL_API_DRAW_BUTTON: u32 = 18;
const SYSCALL_API_DRAW_INPUT: u32 = 19;
const SYSCALL_API_DRAW_PROGRESS: u32 = 20;
const SYSCALL_API_STR_COMPARE: u32 = 21;
const SYSCALL_API_STR_LENGTH: u32 = 22;
const SYSCALL_API_STR_COPY: u32 = 23;
const SYSCALL_API_MEM_COPY: u32 = 24;
const SYSCALL_API_MEM_SET: u32 = 25;
const SYSCALL_API_GET_TIME: u32 = 26;
const SYSCALL_API_GET_DATE: u32 = 27;
const SYSCALL_API_DELAY: u32 = 28;
const SYSCALL_API_FILE_EXISTS: u32 = 29;
const SYSCALL_API_FILE_READ: u32 = 30;
const SYSCALL_API_FILE_SIZE: u32 = 31;
const SYSCALL_API_FILE_WRITE: u32 = 32;
const SYSCALL_API_HAS_KEY: u32 = 33;
const SYSCALL_API_GET_KEY: u32 = 34;
const SYSCALL_API_CLEAR_KEYS: u32 = 35;
const SYSCALL_API_GET_SYSINFO: u32 = 36;
const SYSCALL_API_SET_USERNAME: u32 = 37;
const SYSCALL_API_SET_HOSTNAME: u32 = 38;
const SYSCALL_API_GFX_PIXEL: u32 = 39;
const SYSCALL_API_GFX_LINE: u32 = 40;
const SYSCALL_API_GFX_RECT: u32 = 41;
const SYSCALL_API_GFX_FILLRECT: u32 = 42;
const SYSCALL_API_GFX_CIRCLE: u32 = 43;
const SYSCALL_API_GFX_FILLCIRCLE: u32 = 44;
const SYSCALL_API_GFX_CHAR: u32 = 45;
const SYSCALL_API_GFX_TEXT: u32 = 46;
const SYSCALL_API_GFX_WIDTH: u32 = 47;
const SYSCALL_API_GFX_HEIGHT: u32 = 48;
const SYSCALL_API_GFX_BLIT: u32 = 49;
const SYSCALL_API_GUI_LABEL: u32 = 50;
const SYSCALL_API_GUI_CHECKBOX: u32 = 51;
const SYSCALL_API_GUI_MENUBAR: u32 = 52;
const SYSCALL_API_GUI_TABBAR: u32 = 53;
const SYSCALL_API_GUI_DRAW_CURSOR: u32 = 54;
const SYSCALL_API_GUI_LISTBOX: u32 = 55;
const SYSCALL_API_GUI_SCROLLBAR: u32 = 56;
const SYSCALL_API_GUI_POPUP_MENU: u32 = 57;
const SYSCALL_API_GUI_TITLEBAR: u32 = 58;
const SYSCALL_API_STR_CONCAT: u32 = 59;
const SYSCALL_API_STR_NCOMPARE: u32 = 60;
const SYSCALL_API_STR_TO_INT: u32 = 61;
const SYSCALL_API_INT_TO_STR: u32 = 62;
const SYSCALL_API_MOUSE_GET_STATE: u32 = 63;
const SYSCALL_API_MOUSE_SET_POS: u32 = 64;
const SYSCALL_API_MOUSE_SET_SENS: u32 = 65;

/// Вызов API (INT 0x80, EAX=3): dispatch по таблице 0x100000.
#[no_mangle]
pub unsafe extern "C" fn syscall_api_call(fn_id: u32, a: *const u32) -> u32 {
    use crate::api::{api_table, ApiMouseState, SysInfo};

    const ARG_BYTES: u32 = 8 * core::mem::size_of::<u32>() as u32;

    let t = api_table();
    if user_ptr_ok(a as *const u8, ARG_BYTES) == 0 {
        return 0;
    }

    let mut arg = [0u32; 8];
    ptr::copy_nonoverlapping(a, arg.as_mut_ptr(), 8);
    let table = unsafe { &*t };

    macro_rules! call {
        ($f:ident) => {{
            match table.$f {
                Some(f) => f,
                None => return 0,
            }
        }};
    }

    match fn_id {
        SYSCALL_API_PRINT => {
            if user_str_ok(arg[0] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(print)(arg[0] as *const u8);
            0
        }
        SYSCALL_API_PRINTLN => {
            if user_str_ok(arg[0] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(println)(arg[0] as *const u8);
            0
        }
        SYSCALL_API_PRINT_CHAR => {
            call!(print_char)((arg[0] & 0xFF) as u8);
            0
        }
        SYSCALL_API_PRINT_INT => {
            call!(print_int)(arg[0] as i32);
            0
        }
        SYSCALL_API_PRINT_HEX => {
            call!(print_hex)(arg[0]);
            0
        }
        SYSCALL_API_SET_COLOR => {
            call!(set_color)(arg[0], arg[1]);
            0
        }
        SYSCALL_API_READ_CHAR => call!(read_char)() as u32,
        SYSCALL_API_READ_LINE => {
            if user_ptr_ok(arg[0] as *const u8, if arg[1] != 0 { arg[1] } else { 1 }) == 0 {
                return 0;
            }
            call!(read_line)(arg[0] as *mut u8, arg[1] as i32);
            0
        }
        SYSCALL_API_CLEAR_SCREEN => {
            call!(clear_screen)();
            0
        }
        SYSCALL_API_SET_CURSOR => {
            call!(set_cursor)(arg[0] as i32, arg[1] as i32);
            0
        }
        SYSCALL_API_GET_CURSOR => {
            if user_ptr_ok(arg[0] as *const u8, 4) == 0 || user_ptr_ok(arg[1] as *const u8, 4) == 0 {
                return 0;
            }
            call!(get_cursor)(arg[0] as *mut i32, arg[1] as *mut i32);
            0
        }
        SYSCALL_API_DRAW_BOX => {
            call!(draw_box)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as i32, arg[4], arg[5]);
            0
        }
        SYSCALL_API_DRAW_BOX_TITLED => {
            if user_str_ok(arg[4] as *const u8, 256) == 0 {
                return 0;
            }
            call!(draw_box_titled)(
                arg[0] as i32,
                arg[1] as i32,
                arg[2] as i32,
                arg[3] as i32,
                arg[4] as *const u8,
                arg[5],
                arg[6],
            );
            0
        }
        SYSCALL_API_FILL_RECT => {
            call!(fill_rect)(
                arg[0] as i32,
                arg[1] as i32,
                arg[2] as i32,
                arg[3] as i32,
                (arg[4] & 0xFF) as u8,
                arg[5],
                arg[6],
            );
            0
        }
        SYSCALL_API_DRAW_HLINE => {
            call!(draw_hline)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3], arg[4]);
            0
        }
        SYSCALL_API_DRAW_VLINE => {
            call!(draw_vline)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3], arg[4]);
            0
        }
        SYSCALL_API_DRAW_WINDOW => {
            if user_str_ok(arg[4] as *const u8, 256) == 0 {
                return 0;
            }
            call!(draw_window)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as i32, arg[4] as *const u8);
            0
        }
        SYSCALL_API_DRAW_BUTTON => {
            if user_str_ok(arg[2] as *const u8, 256) == 0 {
                return 0;
            }
            call!(draw_button)(arg[0] as i32, arg[1] as i32, arg[2] as *const u8, arg[3] as i32);
            0
        }
        SYSCALL_API_DRAW_INPUT => {
            if user_str_ok(arg[3] as *const u8, 256) == 0 {
                return 0;
            }
            call!(draw_input)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as *const u8, arg[4] as i32);
            0
        }
        SYSCALL_API_DRAW_PROGRESS => {
            call!(draw_progress)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as i32);
            0
        }
        SYSCALL_API_STR_COMPARE => {
            if user_str_ok(arg[0] as *const u8, 1024) == 0 || user_str_ok(arg[1] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(str_compare)(arg[0] as *const u8, arg[1] as *const u8) as u32
        }
        SYSCALL_API_STR_LENGTH => {
            if user_str_ok(arg[0] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(str_length)(arg[0] as *const u8) as u32
        }
        SYSCALL_API_STR_COPY => {
            if user_ptr_ok(arg[0] as *const u8, 256) == 0 || user_str_ok(arg[1] as *const u8, 256) == 0 {
                return 0;
            }
            call!(str_copy)(arg[0] as *mut u8, arg[1] as *const u8);
            0
        }
        SYSCALL_API_MEM_COPY => {
            if user_ptr_ok(arg[0] as *const u8, arg[2]) == 0 || user_ptr_ok(arg[1] as *const u8, arg[2]) == 0 {
                return 0;
            }
            call!(mem_copy)(arg[0] as *mut u8, arg[1] as *const u8, arg[2]);
            0
        }
        SYSCALL_API_MEM_SET => {
            if user_ptr_ok(arg[0] as *const u8, arg[2]) == 0 {
                return 0;
            }
            call!(mem_set)(arg[0] as *mut u8, (arg[1] & 0xFF) as u8, arg[2]);
            0
        }
        SYSCALL_API_GET_TIME => {
            if user_ptr_ok(arg[0] as *const u8, 1) == 0
                || user_ptr_ok(arg[1] as *const u8, 1) == 0
                || user_ptr_ok(arg[2] as *const u8, 1) == 0
            {
                return 0;
            }
            call!(get_time)(arg[0] as *mut u8, arg[1] as *mut u8, arg[2] as *mut u8);
            0
        }
        SYSCALL_API_GET_DATE => {
            if user_ptr_ok(arg[0] as *const u8, 1) == 0
                || user_ptr_ok(arg[1] as *const u8, 1) == 0
                || user_ptr_ok(arg[2] as *const u8, 2) == 0
            {
                return 0;
            }
            call!(get_date)(arg[0] as *mut u8, arg[1] as *mut u8, arg[2] as *mut u16);
            0
        }
        SYSCALL_API_DELAY => {
            call!(delay)(arg[0]);
            0
        }
        SYSCALL_API_FILE_EXISTS => {
            if user_str_ok(arg[0] as *const u8, 256) == 0 {
                return u32::MAX;
            }
            call!(file_exists)(arg[0] as *const u8) as u32
        }
        SYSCALL_API_FILE_READ => {
            if user_str_ok(arg[0] as *const u8, 256) == 0 || user_ptr_ok(arg[1] as *const u8, arg[2]) == 0 {
                return u32::MAX;
            }
            call!(file_read)(arg[0] as *const u8, arg[1] as *mut u8, arg[2]) as u32
        }
        SYSCALL_API_FILE_SIZE => {
            if user_str_ok(arg[0] as *const u8, 256) == 0 {
                return 0;
            }
            call!(file_size)(arg[0] as *const u8)
        }
        SYSCALL_API_FILE_WRITE => {
            if user_str_ok(arg[0] as *const u8, 256) == 0 || user_ptr_ok(arg[1] as *const u8, arg[2]) == 0 {
                return u32::MAX;
            }
            call!(file_write)(arg[0] as *const u8, arg[1] as *const u8, arg[2]) as u32
        }
        SYSCALL_API_HAS_KEY => call!(has_key)() as u32,
        SYSCALL_API_GET_KEY => call!(get_key)() as u32,
        SYSCALL_API_CLEAR_KEYS => {
            call!(clear_keys)();
            0
        }
        SYSCALL_API_GET_SYSINFO => {
            if user_ptr_ok(arg[0] as *const u8, core::mem::size_of::<SysInfo>() as u32) == 0 {
                return 0;
            }
            call!(get_sysinfo)(arg[0] as *mut SysInfo);
            0
        }
        SYSCALL_API_SET_USERNAME => {
            if user_str_ok(arg[0] as *const u8, 32) == 0 {
                return 0;
            }
            call!(set_username)(arg[0] as *const u8);
            0
        }
        SYSCALL_API_SET_HOSTNAME => {
            if user_str_ok(arg[0] as *const u8, 32) == 0 {
                return 0;
            }
            call!(set_hostname)(arg[0] as *const u8);
            0
        }
        SYSCALL_API_GFX_PIXEL => {
            call!(gfx_pixel)(arg[0] as i32, arg[1] as i32, arg[2]);
            0
        }
        SYSCALL_API_GFX_LINE => {
            call!(gfx_line)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as i32, arg[4]);
            0
        }
        SYSCALL_API_GFX_RECT => {
            call!(gfx_rect)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as i32, arg[4]);
            0
        }
        SYSCALL_API_GFX_FILLRECT => {
            call!(gfx_fillrect)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as i32, arg[4]);
            0
        }
        SYSCALL_API_GFX_CIRCLE => {
            call!(gfx_circle)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3]);
            0
        }
        SYSCALL_API_GFX_FILLCIRCLE => {
            call!(gfx_fillcircle)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3]);
            0
        }
        SYSCALL_API_GFX_CHAR => {
            call!(gfx_char)(arg[0] as i32, arg[1] as i32, (arg[2] & 0xFF) as u8, arg[3], arg[4]);
            0
        }
        SYSCALL_API_GFX_TEXT => {
            if user_str_ok(arg[2] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(gfx_text)(arg[0] as i32, arg[1] as i32, arg[2] as *const u8, arg[3], arg[4]);
            0
        }
        SYSCALL_API_GFX_WIDTH => call!(gfx_width)() as u32,
        SYSCALL_API_GFX_HEIGHT => call!(gfx_height)() as u32,
        SYSCALL_API_GFX_BLIT => {
            let bytes = arg[2]
                .checked_mul(arg[3])
                .and_then(|v| v.checked_mul(core::mem::size_of::<u32>() as u32));
            match bytes {
                Some(b) if user_ptr_ok(arg[4] as *const u8, b) != 0 => {
                    call!(gfx_blit)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as i32, arg[4] as *const u32);
                    0
                }
                _ => 0,
            }
        }
        SYSCALL_API_GUI_LABEL => {
            if user_str_ok(arg[2] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(gui_label)(arg[0] as i32, arg[1] as i32, arg[2] as *const u8, arg[3]);
            0
        }
        SYSCALL_API_GUI_CHECKBOX => {
            if user_str_ok(arg[2] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(gui_checkbox)(arg[0] as i32, arg[1] as i32, arg[2] as *const u8, arg[3] as i32);
            0
        }
        SYSCALL_API_GUI_MENUBAR => {
            call!(gui_menubar)(arg[0] as i32, arg[1] as *const *const u8, arg[2] as i32, arg[3] as i32);
            0
        }
        SYSCALL_API_GUI_TABBAR => {
            call!(gui_tabbar)(
                arg[0] as i32,
                arg[1] as i32,
                arg[2] as i32,
                arg[3] as *const *const u8,
                arg[4] as i32,
                arg[5] as i32,
            );
            0
        }
        SYSCALL_API_GUI_DRAW_CURSOR => {
            call!(gui_draw_cursor)(arg[0] as i32, arg[1] as i32);
            0
        }
        SYSCALL_API_GUI_LISTBOX => {
            call!(gui_listbox)(
                arg[0] as i32,
                arg[1] as i32,
                arg[2] as i32,
                arg[3] as i32,
                arg[4] as *const *const u8,
                arg[5] as i32,
                arg[6] as i32,
            );
            0
        }
        SYSCALL_API_GUI_SCROLLBAR => {
            call!(gui_scrollbar)(
                arg[0] as i32,
                arg[1] as i32,
                arg[2] as i32,
                arg[3] as i32,
                arg[4] as i32,
                arg[5] as i32,
            );
            0
        }
        SYSCALL_API_GUI_POPUP_MENU => {
            call!(gui_popup_menu)(arg[0] as i32, arg[1] as i32, arg[2] as *const *const u8, arg[3] as i32, arg[4] as i32);
            0
        }
        SYSCALL_API_GUI_TITLEBAR => {
            if user_str_ok(arg[3] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(gui_titlebar)(arg[0] as i32, arg[1] as i32, arg[2] as i32, arg[3] as *const u8, arg[4] as i32);
            0
        }
        SYSCALL_API_STR_CONCAT => {
            if user_ptr_ok(arg[0] as *const u8, 256) == 0 || user_str_ok(arg[1] as *const u8, 256) == 0 {
                return 0;
            }
            call!(str_concat)(arg[0] as *mut u8, arg[1] as *const u8);
            0
        }
        SYSCALL_API_STR_NCOMPARE => {
            if user_str_ok(arg[0] as *const u8, 1024) == 0 || user_str_ok(arg[1] as *const u8, 1024) == 0 {
                return 0;
            }
            call!(str_ncompare)(arg[0] as *const u8, arg[1] as *const u8, arg[2] as i32) as u32
        }
        SYSCALL_API_STR_TO_INT => {
            if user_str_ok(arg[0] as *const u8, 64) == 0 {
                return 0;
            }
            call!(str_to_int)(arg[0] as *const u8) as u32
        }
        SYSCALL_API_INT_TO_STR => {
            if user_ptr_ok(arg[1] as *const u8, 16) == 0 {
                return 0;
            }
            call!(int_to_str)(arg[0] as i32, arg[1] as *mut u8);
            0
        }
        SYSCALL_API_MOUSE_GET_STATE => {
            if user_ptr_ok(arg[0] as *const u8, core::mem::size_of::<ApiMouseState>() as u32) == 0 {
                return 0;
            }
            call!(mouse_get_state)(arg[0] as *mut ApiMouseState);
            0
        }
        SYSCALL_API_MOUSE_SET_POS => {
            call!(mouse_set_pos)(arg[0] as i32, arg[1] as i32);
            0
        }
        SYSCALL_API_MOUSE_SET_SENS => {
            call!(mouse_set_sens)(arg[0] as i32, arg[1] as i32);
            0
        }
        _ => {
            let _ = table;
            0
        }
    }
}

// --------------------------------------------------------------------------
// kernel_main — перенос kernel/kernel.c
// --------------------------------------------------------------------------

const DISK_BUF_FLAG_ADDR: u32 = 0x8014;

fn disk_buf_flag() -> u32 {
    unsafe { ptr::read_volatile(DISK_BUF_FLAG_ADDR as *const u32) }
}

fn kstep(msg: *const u8) {
    v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
    v::video_print(b"  [.] \0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    v::video_print(msg);
    v::video_print(b"...\0".as_ptr());
}

fn kok() {
    v::video_set_color(v::COLOR_LIGHT_GREEN, v::COLOR_BLACK);
    v::video_print(b" OK\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

fn kskip(reason: *const u8) {
    v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
    v::video_print(b" SKIP\0".as_ptr());
    if !reason.is_null() {
        v::video_print(b" (\0".as_ptr());
        v::video_print(reason);
        v::video_print(b")\0".as_ptr());
    }
    v::video_print(b"\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

/// Точка входа ядра (порт `kernel_main` из kernel/kernel.c).
#[no_mangle]
pub unsafe extern "C" fn kernel_main() -> ! {
    serial_selftest();
    let ret = v::video_init();
    if ret != 0 {
        // Вывод напрямую через BIOS VGA.
        let vga = 0xB8000 as *mut u8;
        let msg: *const u8 = b"VIDEO INIT FAILED\0".as_ptr();
        let mut i = 0;
        while *msg.add(i) != 0 {
            *vga.add(i * 2) = *msg.add(i);
            *vga.add(i * 2 + 1) = 0x4F; // белый на красном
            i += 1;
        }
        loop {
            core::arch::asm!("hlt", options(nostack));
        }
    }

    v::video_set_color(v::COLOR_LIGHT_GREEN, v::COLOR_BLACK);
    v::video_print(b"Zinux debug checking\n\0".as_ptr());
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);

    kstep(b"memory\0".as_ptr());
    memory_init();
    kok();

    kstep(b"system\0".as_ptr());
    system_init();
    kok();

    kstep(b"keyboard\0".as_ptr());
    keyboard_init();
    kok();

    kstep(b"mouse\0".as_ptr());
    mouse_init(v::video_width(), v::video_height());
    if mouse_is_ready() != 0 {
        kok();
    } else {
        kskip(b"init failed\0".as_ptr());
    }

    kstep(b"IRQ\0".as_ptr());
    enable_interrupts();
    kok();

    kstep(b"PIT\0".as_ptr());
    pit_init(PIT_TARGET_HZ);
    kok();

    kstep(b"disk\0".as_ptr());
    ata_init();
    if disk_buf_flag() > 0 {
        v::video_set_color(v::COLOR_LIGHT_GREEN, v::COLOR_BLACK);
        v::video_print(b" OK (RAM disk, \0".as_ptr());
        v::video_put_dec(disk_buf_flag() / 2);
        v::video_print(b"KB)\n\0".as_ptr());
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    } else {
        kskip(b"no BIOS preload, trying ATA PIO\0".as_ptr());
    }

    kstep(b"FAT16\0".as_ptr());
    if fat16_init(2048) == 0 {
        shell_set_fs_ready(1);
        kok();
        shell_load_config();
    } else {
        kskip(b"no disk\0".as_ptr());
    }

    kstep(b"API\0".as_ptr());
    crate::api::api_setup_table();
    kok();

    kstep(b"ZXE\0".as_ptr());
    crate::zxe::zxe_init();
    kok();

    kstep(b"shell\0".as_ptr());
    shell_init();
    kok();

    let dirty = ata_dirty_count();
    if dirty > 0 {
        v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
        v::video_print(b"  [~] Syncing init data...\0".as_ptr());
        if ata_flush() == 0 {
            v::video_print(b" OK\n\0".as_ptr());
        } else {
            v::video_print(b" ERR\n\0".as_ptr());
        }
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    }

    v::video_print(b"\n\0".as_ptr());
    shell_run();

    let dirty = ata_dirty_count();
    if dirty > 0 {
        v::video_set_color(v::COLOR_YELLOW, v::COLOR_BLACK);
        v::video_print(b"  [~] Flushing \0".as_ptr());
        v::video_put_dec(dirty as u32);
        v::video_print(b" sectors...\0".as_ptr());
        let r = ata_flush();
        if r == 0 {
            v::video_set_color(v::COLOR_LIGHT_GREEN, v::COLOR_BLACK);
            v::video_print(b" OK\n\0".as_ptr());
        } else {
            v::video_set_color(v::COLOR_LIGHT_RED, v::COLOR_BLACK);
            v::video_print(b" ERRORS\n\0".as_ptr());
        }
        v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    }

    system_halt();
}