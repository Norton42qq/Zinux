//! Перенос `kernel/api.c` + `kernel/api.h` — таблица API для пользовательских
//! программ (65 функций по адресу 0x100000). Вызывается только ядром через
//! `system::syscall_api_call` (INT 0x80, EAX=3, EBX=id, ECX=args).

use core::ptr;

use zlibk::mem::{memcpy, memset};
use zlibk::num::{atoi, itoa};
use zlibk::str::{strcat, strcmp, strcpy, strlen, strncmp, strncpy};

use crate::arch::io::{inb, outb};
use crate::drivers::ata::ata_flush;
use crate::drivers::keyboard::{
    keyboard_clear_buffer, keyboard_getchar, keyboard_haschar, keyboard_wait_char,
};
use crate::drivers::mouse::{mouse_get_state, mouse_set_pos, mouse_set_sensitivity};
use crate::drivers::video as v;
use crate::fat16::{fat16_file_exists, fat16_file_size, fat16_read_file, fat16_write_file};
use crate::shell::{shell_get_cwd, shell_save_config};

pub const API_TABLE_ADDRESS: u32 = 0x100000;

/// Системная информация (`sys_info_t` из api.h).
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SysInfo {
    pub username: [u8; 32],
    pub hostname: [u8; 32],
    pub os_name: [u8; 32],
    pub os_version: [u8; 16],
    pub cwd: [u8; 64],
    pub mem_kb: u32,
}

/// Состояние мыши для пользователя (`mouse_state_t` из api.h, packed).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ApiMouseState {
    pub x: i32,
    pub y: i32,
    pub buttons: u8,
    pub wheel: i32,
}

// Типы указателей на функции таблицы.
type FnPrint = unsafe extern "C" fn(*const u8);
type FnPrintln = unsafe extern "C" fn(*const u8);
type FnPrintChar = unsafe extern "C" fn(u8);
type FnPrintInt = unsafe extern "C" fn(i32);
type FnPrintHex = unsafe extern "C" fn(u32);
type FnSetColor = unsafe extern "C" fn(u32, u32);
type FnReadChar = unsafe extern "C" fn() -> u8;
type FnReadLine = unsafe extern "C" fn(*mut u8, i32);
type FnClearScreen = unsafe extern "C" fn();
type FnSetCursor = unsafe extern "C" fn(i32, i32);
type FnGetCursor = unsafe extern "C" fn(*mut i32, *mut i32);
type FnDrawBox = unsafe extern "C" fn(i32, i32, i32, i32, u32, u32);
type FnDrawBoxTitled = unsafe extern "C" fn(i32, i32, i32, i32, *const u8, u32, u32);
type FnFillRect = unsafe extern "C" fn(i32, i32, i32, i32, u8, u32, u32);
type FnDrawHline = unsafe extern "C" fn(i32, i32, i32, u32, u32);
type FnDrawVline = unsafe extern "C" fn(i32, i32, i32, u32, u32);
type FnDrawWindow = unsafe extern "C" fn(i32, i32, i32, i32, *const u8);
type FnDrawButton = unsafe extern "C" fn(i32, i32, *const u8, i32);
type FnDrawInput = unsafe extern "C" fn(i32, i32, i32, *const u8, i32);
type FnDrawProgress = unsafe extern "C" fn(i32, i32, i32, i32);
type FnStrCompare = unsafe extern "C" fn(*const u8, *const u8) -> i32;
type FnStrLength = unsafe extern "C" fn(*const u8) -> i32;
type FnStrCopy = unsafe extern "C" fn(*mut u8, *const u8);
type FnMemCopy = unsafe extern "C" fn(*mut u8, *const u8, u32);
type FnMemSet = unsafe extern "C" fn(*mut u8, u8, u32);
type FnGetTime = unsafe extern "C" fn(*mut u8, *mut u8, *mut u8);
type FnGetDate = unsafe extern "C" fn(*mut u8, *mut u8, *mut u16);
type FnDelay = unsafe extern "C" fn(u32);
type FnFileExists = unsafe extern "C" fn(*const u8) -> i32;
type FnFileRead = unsafe extern "C" fn(*const u8, *mut u8, u32) -> i32;
type FnFileSize = unsafe extern "C" fn(*const u8) -> u32;
type FnFileWrite = unsafe extern "C" fn(*const u8, *const u8, u32) -> i32;
type FnHasKey = unsafe extern "C" fn() -> i32;
type FnGetKey = unsafe extern "C" fn() -> u8;
type FnClearKeys = unsafe extern "C" fn();
type FnGetSysinfo = unsafe extern "C" fn(*mut SysInfo);
type FnSetUsername = unsafe extern "C" fn(*const u8);
type FnSetHostname = unsafe extern "C" fn(*const u8);
type FnGfxPixel = unsafe extern "C" fn(i32, i32, u32);
type FnGfxLine = unsafe extern "C" fn(i32, i32, i32, i32, u32);
type FnGfxRect = unsafe extern "C" fn(i32, i32, i32, i32, u32);
type FnGfxFillRect = unsafe extern "C" fn(i32, i32, i32, i32, u32);
type FnGfxCircle = unsafe extern "C" fn(i32, i32, i32, u32);
type FnGfxFillCircle = unsafe extern "C" fn(i32, i32, i32, u32);
type FnGfxChar = unsafe extern "C" fn(i32, i32, u8, u32, u32);
type FnGfxText = unsafe extern "C" fn(i32, i32, *const u8, u32, u32);
type FnGfxWidth = unsafe extern "C" fn() -> i32;
type FnGfxHeight = unsafe extern "C" fn() -> i32;
type FnGfxBlit = unsafe extern "C" fn(i32, i32, i32, i32, *const u32);
type FnGuiLabel = unsafe extern "C" fn(i32, i32, *const u8, u32);
type FnGuiCheckbox = unsafe extern "C" fn(i32, i32, *const u8, i32);
type FnGuiMenubar = unsafe extern "C" fn(i32, *const *const u8, i32, i32);
type FnGuiTabbar = unsafe extern "C" fn(i32, i32, i32, *const *const u8, i32, i32);
type FnGuiDrawCursor = unsafe extern "C" fn(i32, i32);
type FnGuiListbox = unsafe extern "C" fn(i32, i32, i32, i32, *const *const u8, i32, i32);
type FnGuiScrollbar = unsafe extern "C" fn(i32, i32, i32, i32, i32, i32);
type FnGuiPopupMenu = unsafe extern "C" fn(i32, i32, *const *const u8, i32, i32);
type FnGuiTitlebar = unsafe extern "C" fn(i32, i32, i32, *const u8, i32);
type FnStrConcat = unsafe extern "C" fn(*mut u8, *const u8);
type FnStrNcompare = unsafe extern "C" fn(*const u8, *const u8, i32) -> i32;
type FnStrToInt = unsafe extern "C" fn(*const u8) -> i32;
type FnIntToStr = unsafe extern "C" fn(i32, *mut u8);
type FnMouseGetState = unsafe extern "C" fn(*mut ApiMouseState);
type FnMouseSetPos = unsafe extern "C" fn(i32, i32);
type FnMouseSetSens = unsafe extern "C" fn(i32, i32);

/// Таблица API (`api_table_t`, 65 указателей, packed в C — все 4 байта).
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ApiTable {
    pub print: Option<FnPrint>,
    pub println: Option<FnPrintln>,
    pub print_char: Option<FnPrintChar>,
    pub print_int: Option<FnPrintInt>,
    pub print_hex: Option<FnPrintHex>,
    pub set_color: Option<FnSetColor>,
    pub read_char: Option<FnReadChar>,
    pub read_line: Option<FnReadLine>,
    pub clear_screen: Option<FnClearScreen>,
    pub set_cursor: Option<FnSetCursor>,
    pub get_cursor: Option<FnGetCursor>,
    pub draw_box: Option<FnDrawBox>,
    pub draw_box_titled: Option<FnDrawBoxTitled>,
    pub fill_rect: Option<FnFillRect>,
    pub draw_hline: Option<FnDrawHline>,
    pub draw_vline: Option<FnDrawVline>,
    pub draw_window: Option<FnDrawWindow>,
    pub draw_button: Option<FnDrawButton>,
    pub draw_input: Option<FnDrawInput>,
    pub draw_progress: Option<FnDrawProgress>,
    pub str_compare: Option<FnStrCompare>,
    pub str_length: Option<FnStrLength>,
    pub str_copy: Option<FnStrCopy>,
    pub mem_copy: Option<FnMemCopy>,
    pub mem_set: Option<FnMemSet>,
    pub get_time: Option<FnGetTime>,
    pub get_date: Option<FnGetDate>,
    pub delay: Option<FnDelay>,
    pub file_exists: Option<FnFileExists>,
    pub file_read: Option<FnFileRead>,
    pub file_size: Option<FnFileSize>,
    pub file_write: Option<FnFileWrite>,
    pub has_key: Option<FnHasKey>,
    pub get_key: Option<FnGetKey>,
    pub clear_keys: Option<FnClearKeys>,
    pub get_sysinfo: Option<FnGetSysinfo>,
    pub set_username: Option<FnSetUsername>,
    pub set_hostname: Option<FnSetHostname>,
    pub gfx_pixel: Option<FnGfxPixel>,
    pub gfx_line: Option<FnGfxLine>,
    pub gfx_rect: Option<FnGfxRect>,
    pub gfx_fillrect: Option<FnGfxFillRect>,
    pub gfx_circle: Option<FnGfxCircle>,
    pub gfx_fillcircle: Option<FnGfxFillCircle>,
    pub gfx_char: Option<FnGfxChar>,
    pub gfx_text: Option<FnGfxText>,
    pub gfx_width: Option<FnGfxWidth>,
    pub gfx_height: Option<FnGfxHeight>,
    pub gfx_blit: Option<FnGfxBlit>,
    pub gui_label: Option<FnGuiLabel>,
    pub gui_checkbox: Option<FnGuiCheckbox>,
    pub gui_menubar: Option<FnGuiMenubar>,
    pub gui_tabbar: Option<FnGuiTabbar>,
    pub gui_draw_cursor: Option<FnGuiDrawCursor>,
    pub gui_listbox: Option<FnGuiListbox>,
    pub gui_scrollbar: Option<FnGuiScrollbar>,
    pub gui_popup_menu: Option<FnGuiPopupMenu>,
    pub gui_titlebar: Option<FnGuiTitlebar>,
    pub str_concat: Option<FnStrConcat>,
    pub str_ncompare: Option<FnStrNcompare>,
    pub str_to_int: Option<FnStrToInt>,
    pub int_to_str: Option<FnIntToStr>,
    pub mouse_get_state: Option<FnMouseGetState>,
    pub mouse_set_pos: Option<FnMouseSetPos>,
    pub mouse_set_sens: Option<FnMouseSetSens>,
}

/// Доступ к таблице API (для `system::syscall_api_call`).
pub fn api_table() -> *const ApiTable {
    API_TABLE_ADDRESS as *const ApiTable
}

// --------------------------------------------------------------------------
// Текст
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_print(s: *const u8) {
    v::video_print(s);
}

unsafe extern "C" fn ap_println(s: *const u8) {
    v::video_print(s);
    v::video_print_char(b'\n');
}

unsafe extern "C" fn ap_print_char(c: u8) {
    v::video_print_char(c);
}

unsafe extern "C" fn ap_print_hex(val: u32) {
    v::video_put_hex(val);
}

unsafe extern "C" fn ap_print_int(mut val: i32) {
    if val < 0 {
        v::video_print_char(b'-');
        val = -val;
    }
    v::video_put_dec(val as u32);
}

unsafe extern "C" fn ap_set_color(fg: u32, bg: u32) {
    v::video_set_color(fg, bg);
}

// --------------------------------------------------------------------------
// Ввод
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_read_char() -> u8 {
    keyboard_wait_char()
}

unsafe extern "C" fn ap_read_line(buf: *mut u8, max_len: i32) {
    let mut pos: usize = 0;
    while (pos as i32) < max_len - 1 {
        let c = keyboard_wait_char();
        if c == b'\n' {
            v::video_print_char(b'\n');
            break;
        }
        if c == b'\x08' {
            if pos > 0 {
                pos -= 1;
                v::video_backspace();
            }
            continue;
        }
        if c >= 32 {
            *buf.add(pos) = c;
            pos += 1;
            v::video_print_char(c);
        }
    }
    *buf.add(pos) = 0;
}

// --------------------------------------------------------------------------
// Экран
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_clear() {
    v::video_clear(v::COLOR_BLACK);
}

unsafe extern "C" fn ap_set_cur(x: i32, y: i32) {
    v::video_set_cursor(x, y);
}

unsafe extern "C" fn ap_get_cur(x: *mut i32, y: *mut i32) {
    v::video_get_cursor(x, y);
}

// --------------------------------------------------------------------------
// TUI: рамки
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_draw_box(x: i32, y: i32, w: i32, h: i32, fg: u32, bg: u32) {
    v::video_set_color(fg, bg);

    v::video_set_cursor(x, y);
    v::video_print_char(0xDA); // ┌
    v::video_set_cursor(x + w - 1, y);
    v::video_print_char(0xBF); // ┐
    v::video_set_cursor(x, y + h - 1);
    v::video_print_char(0xC0); // └
    v::video_set_cursor(x + w - 1, y + h - 1);
    v::video_print_char(0xD9); // ┘

    let mut i = 1;
    while i < w - 1 {
        v::video_set_cursor(x + i, y);
        v::video_print_char(0xC4);
        v::video_set_cursor(x + i, y + h - 1);
        v::video_print_char(0xC4);
        i += 1;
    }

    let mut j = 1;
    while j < h - 1 {
        v::video_set_cursor(x, y + j);
        v::video_print_char(0xB3); // │
        v::video_set_cursor(x + w - 1, y + j);
        v::video_print_char(0xB3);
        j += 1;
    }
}

unsafe extern "C" fn ap_draw_box_titled(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    title: *const u8,
    fg: u32,
    bg: u32,
) {
    ap_draw_box(x, y, w, h, fg, bg);

    if !title.is_null() && *title != 0 {
        let title_len = strlen(title) as i32;
        let title_x = x + (w - title_len - 2) / 2;

        v::video_set_cursor(title_x, y);
        v::video_set_color(fg, bg);
        v::video_print_char(b' ');
        v::video_print(title);
        v::video_print_char(b' ');
    }
}

unsafe extern "C" fn ap_fill_rect(x: i32, y: i32, w: i32, h: i32, ch: u8, fg: u32, bg: u32) {
    v::video_set_color(fg, bg);
    let mut row = 0;
    while row < h {
        v::video_set_cursor(x, y + row);
        let mut col = 0;
        while col < w {
            v::video_print_char(ch);
            col += 1;
        }
        row += 1;
    }
}

unsafe extern "C" fn ap_draw_hline(x: i32, y: i32, len: i32, fg: u32, bg: u32) {
    v::video_set_color(fg, bg);
    v::video_set_cursor(x, y);
    let mut i = 0;
    while i < len {
        v::video_print_char(0xC4);
        i += 1;
    }
}

unsafe extern "C" fn ap_draw_vline(x: i32, y: i32, len: i32, fg: u32, bg: u32) {
    v::video_set_color(fg, bg);
    let mut i = 0;
    while i < len {
        v::video_set_cursor(x, y + i);
        v::video_print_char(0xB3); // │
        i += 1;
    }
}

// --------------------------------------------------------------------------
// TUI: окна
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_draw_window(x: i32, y: i32, w: i32, h: i32, title: *const u8) {
    ap_fill_rect(x + 2, y + 1, w, h, 0xB0, v::COLOR_DARK_GREY, v::COLOR_BLACK);
    ap_fill_rect(x, y, w, h, b' ', v::COLOR_WHITE, v::COLOR_BLUE);
    ap_draw_box_titled(x, y, w, h, title, v::COLOR_WHITE, v::COLOR_BLUE);
}

unsafe extern "C" fn ap_draw_button(x: i32, y: i32, text: *const u8, selected: i32) {
    let fg = if selected != 0 { v::COLOR_BLACK } else { v::COLOR_WHITE };
    let bg = if selected != 0 { v::COLOR_WHITE } else { v::COLOR_BLUE };

    v::video_set_color(fg, bg);
    v::video_set_cursor(x, y);
    v::video_print(b"[ \0".as_ptr());
    v::video_print(text);
    v::video_print(b" ]\0".as_ptr());
}

unsafe extern "C" fn ap_draw_input(x: i32, y: i32, w: i32, text: *const u8, active: i32) {
    let (fg, bg) = if active != 0 {
        (v::COLOR_BLACK, v::COLOR_LIGHT_GREY)
    } else {
        (v::COLOR_LIGHT_GREY, v::COLOR_BLACK)
    };

    v::video_set_color(fg, bg);
    v::video_set_cursor(x, y);

    let text_len = strlen(text) as i32;
    v::video_print(text);
    let mut i = text_len;
    while i < w {
        v::video_print_char(b' ');
        i += 1;
    }

    if active != 0 {
        v::video_set_cursor(x + text_len, y);
    }
}

unsafe extern "C" fn ap_draw_progress(x: i32, y: i32, w: i32, percent: i32) {
    v::video_set_cursor(x, y);
    v::video_set_color(v::COLOR_LIGHT_GREY, v::COLOR_BLACK);
    v::video_print_char(b'[');

    let filled = (w - 2) * percent / 100;
    let mut i = 0;
    while i < w - 2 {
        if i < filled {
            v::video_set_color(v::COLOR_WHITE, v::COLOR_GREEN);
            v::video_print_char(0xDB); // █
        } else {
            v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
            v::video_print_char(0xB0); // ░
        }
        i += 1;
    }

    v::video_set_color(v::COLOR_LIGHT_GREY, v::COLOR_BLACK);
    v::video_print_char(b']');
    v::video_print_char(b' ');
    v::video_put_dec(percent as u32);
    v::video_print_char(b'%');
}

// --------------------------------------------------------------------------
// Строки
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_strcmp(a: *const u8, b: *const u8) -> i32 {
    strcmp(a, b)
}

unsafe extern "C" fn ap_strlen(s: *const u8) -> i32 {
    strlen(s) as i32
}

unsafe extern "C" fn ap_strcpy(d: *mut u8, s: *const u8) {
    strcpy(d, s);
}

unsafe extern "C" fn ap_memcpy(d: *mut u8, s: *const u8, n: u32) {
    memcpy(d, s, n as usize);
}

unsafe extern "C" fn ap_memset(d: *mut u8, val: u8, n: u32) {
    memset(d, val as i32, n as usize);
}

// --------------------------------------------------------------------------
// Время
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_get_time(h: *mut u8, m: *mut u8, s: *mut u8) {
    outb(0x70, 0x04);
    let bh = inb(0x71);
    outb(0x70, 0x02);
    let bm = inb(0x71);
    outb(0x70, 0x00);
    let bs = inb(0x71);
    if !h.is_null() {
        *h = ((bh >> 4) * 10) + (bh & 0xF);
    }
    if !m.is_null() {
        *m = ((bm >> 4) * 10) + (bm & 0xF);
    }
    if !s.is_null() {
        *s = ((bs >> 4) * 10) + (bs & 0xF);
    }
}

unsafe extern "C" fn ap_get_date(d: *mut u8, m: *mut u8, y: *mut u16) {
    outb(0x70, 0x07);
    let bd = inb(0x71);
    outb(0x70, 0x08);
    let bm = inb(0x71);
    outb(0x70, 0x09);
    let by = inb(0x71);
    if !d.is_null() {
        *d = ((bd >> 4) * 10) + (bd & 0xF);
    }
    if !m.is_null() {
        *m = ((bm >> 4) * 10) + (bm & 0xF);
    }
    if !y.is_null() {
        *y = 2000 + (((by >> 4) * 10) + (by & 0xF)) as u16;
    }
}

unsafe extern "C" fn ap_delay(ms: u32) {
    let mut i = 0u32;
    while i < ms * 1000 {
        core::arch::asm!("nop", options(nostack));
        i += 1;
    }
}

// --------------------------------------------------------------------------
// Файлы
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_fexists(f: *const u8) -> i32 {
    fat16_file_exists(f)
}

unsafe extern "C" fn ap_fread(f: *const u8, b: *mut u8, mx: u32) -> i32 {
    fat16_read_file(f, b, mx)
}

unsafe extern "C" fn ap_fsize(f: *const u8) -> u32 {
    fat16_file_size(f)
}

unsafe extern "C" fn ap_fwrite(f: *const u8, b: *const u8, n: u32) -> i32 {
    let r = fat16_write_file(f, b, n);
    if r > 0 {
        ata_flush();
    }
    r
}

// --------------------------------------------------------------------------
// Клавиатура
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_haskey() -> i32 {
    keyboard_haschar()
}

unsafe extern "C" fn ap_getkey() -> u8 {
    keyboard_getchar()
}

unsafe extern "C" fn ap_clearkeys() {
    keyboard_clear_buffer();
}

// --------------------------------------------------------------------------
// Системная информация
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_get_sysinfo(info: *mut SysInfo) {
    if info.is_null() {
        return;
    }
    let si = crate::arch::cpu::system_get_info();
    strcpy((*info).username.as_mut_ptr(), crate::shell::CURRENT_CONFIG.username.as_ptr());
    strcpy((*info).hostname.as_mut_ptr(), crate::shell::CURRENT_CONFIG.hostname.as_ptr());
    strcpy((*info).os_name.as_mut_ptr(), b"Zinux\0".as_ptr());
    strcpy((*info).os_version.as_mut_ptr(), b"0.2\0".as_ptr());
    strcpy((*info).cwd.as_mut_ptr(), shell_get_cwd());
    (*info).mem_kb = (*si).mem_lower + (*si).mem_upper;
}

unsafe extern "C" fn ap_set_username(name: *const u8) {
    if name.is_null() {
        return;
    }
    strncpy(crate::shell::CURRENT_CONFIG.username.as_mut_ptr(), name, 31);
    crate::shell::CURRENT_CONFIG.username[31] = 0;
    shell_save_config();
}

unsafe extern "C" fn ap_set_hostname(name: *const u8) {
    if name.is_null() {
        return;
    }
    strncpy(crate::shell::CURRENT_CONFIG.hostname.as_mut_ptr(), name, 31);
    crate::shell::CURRENT_CONFIG.hostname[31] = 0;
    shell_save_config();
}

// --------------------------------------------------------------------------
// Пиксельная графика
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_gfx_pixel(x: i32, y: i32, color: u32) {
    v::video_pixel(x, y, color);
}

unsafe extern "C" fn ap_gfx_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    v::video_line(x0, y0, x1, y1, color);
}

unsafe extern "C" fn ap_gfx_rect(x: i32, y: i32, w: i32, h: i32, color: u32) {
    v::video_rect(x, y, w, h, color);
}

unsafe extern "C" fn ap_gfx_fillrect(x: i32, y: i32, w: i32, h: i32, color: u32) {
    v::video_fillrect(x, y, w, h, color);
}

unsafe extern "C" fn ap_gfx_circle(cx: i32, cy: i32, r: i32, color: u32) {
    v::video_circle(cx, cy, r, color);
}

unsafe extern "C" fn ap_gfx_fillcircle(cx: i32, cy: i32, r: i32, color: u32) {
    v::video_fillcircle(cx, cy, r, color);
}

unsafe extern "C" fn ap_gfx_char(x: i32, y: i32, c: u8, fg: u32, bg: u32) {
    v::video_putchar(x / 8, y / 16, c, fg, bg);
}

unsafe extern "C" fn ap_gfx_text(x: i32, y: i32, s: *const u8, fg: u32, bg: u32) {
    v::video_puts(x / 8, y / 16, s, fg, bg);
}

unsafe extern "C" fn ap_gfx_width() -> i32 {
    v::video_width()
}

unsafe extern "C" fn ap_gfx_height() -> i32 {
    v::video_height()
}

unsafe extern "C" fn ap_gfx_blit(x: i32, y: i32, w: i32, h: i32, pixels: *const u32) {
    let vi = v::video_get_info();
    let mut row = 0;
    while row < h {
        let mut col = 0;
        while col < w {
            let px = x + col;
            let py = y + row;
            if (px as u32) < (*vi).width as u32 && (py as u32) < (*vi).height as u32 {
                v::video_pixel(px, py, *pixels.add((row as usize) * (w as usize) + (col as usize)));
            }
            col += 1;
        }
        row += 1;
    }
}

// --------------------------------------------------------------------------
// GUI
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_gui_label(x: i32, y: i32, text: *const u8, color: u32) {
    v::video_set_cursor(x, y);
    v::video_set_color(color, v::COLOR_BLACK);
    v::video_print(text);
}

unsafe extern "C" fn ap_gui_checkbox(x: i32, y: i32, label: *const u8, checked: i32) {
    v::video_set_cursor(x, y);
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    v::video_print_char(b'[');
    v::video_set_color(
        if checked != 0 { v::COLOR_LIGHT_GREEN } else { v::COLOR_DARK_GREY },
        v::COLOR_BLACK,
    );
    v::video_print_char(if checked != 0 { 0xFB } else { b' ' }); // ✓ или пробел
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
    v::video_print_char(b']');
    v::video_print_char(b' ');
    v::video_print(label);
}

unsafe extern "C" fn ap_gui_menubar(y: i32, items: *const *const u8, count: i32, selected: i32) {
    v::video_set_cursor(0, y);
    v::video_set_color(v::COLOR_BLACK, v::COLOR_LIGHT_GREY);
    let mut i = 0;
    while i < 80 {
        v::video_print_char(b' ');
        i += 1;
    }
    v::video_set_cursor(0, y);
    let mut i = 0;
    while i < count {
        if i == selected {
            v::video_set_color(v::COLOR_WHITE, v::COLOR_BLUE);
        } else {
            v::video_set_color(v::COLOR_BLACK, v::COLOR_LIGHT_GREY);
        }
        v::video_print_char(b' ');
        v::video_print(*items.add(i as usize));
        v::video_print_char(b' ');
        i += 1;
    }
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

unsafe extern "C" fn ap_gui_tabbar(
    x: i32,
    y: i32,
    w: i32,
    tabs: *const *const u8,
    count: i32,
    active: i32,
) {
    let mut cx = x;
    let mut i = 0;
    while i < count {
        let tlen = strlen(*tabs.add(i as usize)) as i32 + 2;
        if i == active {
            v::video_set_color(v::COLOR_WHITE, v::COLOR_BLUE);
        } else {
            v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
        }
        v::video_set_cursor(cx, y);
        v::video_print_char(b' ');
        v::video_print(*tabs.add(i as usize));
        v::video_print_char(b' ');
        if i != active {
            v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
            v::video_set_cursor(cx, y + 1);
            let mut j = 0;
            while j < tlen {
                v::video_print_char(0xC4);
                j += 1;
            }
        }
        cx += tlen;
        i += 1;
    }
    v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
    v::video_set_cursor(cx, y + 1);
    let mut j = cx;
    while j < x + w {
        v::video_print_char(0xC4);
        j += 1;
    }
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

/// Курсор мыши: стрелка 8x12 (XOR-режим).
unsafe fn cursor_xor_pixel(px: i32, py: i32) {
    let vi = v::video_get_info();
    if vi.is_null() {
        return;
    }
    if (px as u32) >= (*vi).width as u32 || (py as u32) >= (*vi).height as u32 {
        return;
    }
    let old = v::video_getpixel(px, py);
    v::video_pixel(px, py, old ^ 0x00FF_FFFF);
}

unsafe extern "C" fn ap_gui_draw_cursor(x: i32, y: i32) {
    let mut i = 0;
    while i < 12 {
        cursor_xor_pixel(x, y + i);
        cursor_xor_pixel(x + 1, y + i);
        i += 1;
    }
    let mut i = 0;
    while i < 12 {
        if i < 12 {
            cursor_xor_pixel(x, y + i);
        }
        if i < 10 {
            cursor_xor_pixel(x + 1, y + i);
        }
        if i < 8 {
            cursor_xor_pixel(x + 2, y + i);
        }
        if i < 6 {
            cursor_xor_pixel(x + 3, y + i);
        }
        if i < 4 {
            cursor_xor_pixel(x + 4, y + i);
        }
        if i < 2 {
            cursor_xor_pixel(x + 5, y + i);
        }
        i += 1;
    }
    cursor_xor_pixel(x + 2, y + 8);
    cursor_xor_pixel(x + 3, y + 9);
    cursor_xor_pixel(x + 2, y + 9);
    cursor_xor_pixel(x + 3, y + 10);
    cursor_xor_pixel(x + 4, y + 10);
    cursor_xor_pixel(x + 3, y + 11);
    cursor_xor_pixel(x + 4, y + 11);
    cursor_xor_pixel(x + 5, y + 11);
    cursor_xor_pixel(x + 1, y + 8);
    cursor_xor_pixel(x + 2, y + 7);
    cursor_xor_pixel(x + 3, y + 8);
    cursor_xor_pixel(x + 2, y + 10);
    cursor_xor_pixel(x + 4, y + 9);
    cursor_xor_pixel(x + 5, y + 10);
    cursor_xor_pixel(x + 6, y + 11);
}

unsafe extern "C" fn ap_gui_listbox(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    items: *const *const u8,
    count: i32,
    selected: i32,
) {
    let mut i = 0;
    while i < h {
        let idx = i;
        if idx < count {
            let (fg, bg) = if idx == selected {
                (v::COLOR_BLACK, v::COLOR_LIGHT_CYAN)
            } else {
                (v::COLOR_WHITE, v::COLOR_BLACK)
            };
            v::video_set_color(fg, bg);
            v::video_set_cursor(x, y + i);
            let item = *items.add(idx as usize);
            let len = strlen(item) as i32;
            let mut j = 0;
            while j < len && j < w {
                v::video_print_char(*item.add(j as usize));
                j += 1;
            }
            while j < w {
                v::video_print_char(b' ');
                j += 1;
            }
        } else {
            v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
            v::video_set_cursor(x, y + i);
            let mut j = 0;
            while j < w {
                v::video_print_char(b' ');
                j += 1;
            }
        }
        i += 1;
    }
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

/// Вертикальный скроллбар, координаты символьные.
unsafe extern "C" fn ap_gui_scrollbar(x: i32, y: i32, h: i32, total: i32, visible: i32, pos: i32) {
    v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
    v::video_set_cursor(x, y);
    v::video_print_char(0x1E); // ▲
    let mut i = 1;
    while i < h - 1 {
        v::video_set_cursor(x, y + i);
        v::video_print_char(0xB0); // ░
        i += 1;
    }
    v::video_set_cursor(x, y + h - 1);
    v::video_print_char(0x1F); // ▼

    if total > visible && h > 2 {
        let track = h - 2;
        let mut thumb_h = track * visible / total;
        if thumb_h < 1 {
            thumb_h = 1;
        }
        let thumb_y = 1 + (track - thumb_h) * pos / (total - visible);
        v::video_set_color(v::COLOR_LIGHT_GREY, v::COLOR_DARK_GREY);
        let mut i = 0;
        while i < thumb_h {
            v::video_set_cursor(x, y + thumb_y + i);
            v::video_print_char(0xDB); // █
            i += 1;
        }
    }
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

/// Выпадающее контекстное меню, координаты символьные.
unsafe extern "C" fn ap_gui_popup_menu(
    x: i32,
    y: i32,
    items: *const *const u8,
    count: i32,
    selected: i32,
) {
    let mut w = 6;
    let mut i = 0;
    while i < count {
        let l = strlen(*items.add(i as usize)) as i32 + 4;
        if l > w {
            w = l;
        }
        i += 1;
    }

    v::video_set_color(v::COLOR_DARK_GREY, v::COLOR_BLACK);
    let mut i = 0;
    while i < count + 2 {
        v::video_set_cursor(x + w, y + 1 + i);
        v::video_print_char(b' ');
        v::video_set_cursor(x + w + 1, y + 1 + i);
        v::video_print_char(b' ');
        i += 1;
    }

    ap_draw_box(x, y, w, count + 2, v::COLOR_WHITE, v::COLOR_BLACK);

    let mut i = 0;
    while i < count {
        let (fg, bg) = if i == selected {
            (v::COLOR_BLACK, v::COLOR_LIGHT_GREY)
        } else {
            (v::COLOR_WHITE, v::COLOR_BLACK)
        };
        v::video_set_color(fg, bg);
        v::video_set_cursor(x + 1, y + 1 + i);
        v::video_print_char(b' ');
        let item = *items.add(i as usize);
        let len = strlen(item) as i32;
        let mut j = 0;
        while j < len && j < w - 3 {
            v::video_print_char(*item.add(j as usize));
            j += 1;
        }
        while j < w - 3 {
            v::video_print_char(b' ');
            j += 1;
        }
        v::video_print_char(b' ');
        i += 1;
    }
    v::video_set_color(v::COLOR_WHITE, v::COLOR_BLACK);
}

/// Заголовок окна (пиксельная графика).
unsafe extern "C" fn ap_gui_titlebar(x: i32, y: i32, w: i32, title: *const u8, active: i32) {
    let h = 18;
    let bg = if active != 0 { v::rgb(0, 0, 128) } else { v::rgb(80, 80, 80) };
    let light = if active != 0 { v::rgb(0, 0, 200) } else { v::rgb(110, 110, 110) };
    let dark = if active != 0 { v::rgb(0, 0, 80) } else { v::rgb(50, 50, 50) };

    v::video_fillrect(x, y, w, 6, light);
    v::video_fillrect(x, y + 6, w, 6, bg);
    v::video_fillrect(x, y + 12, w, h - 12, dark);

    let tx = x + 4;
    let ty = y + 1;
    v::video_puts(tx / 8, ty / 16, title, v::COLOR_WHITE, bg);

    let bx = x + w - 18;
    let by = y + 2;
    v::video_fillrect(bx, by, 14, 14, v::rgb(180, 40, 40));
    v::video_rect(bx, by, 14, 14, v::rgb(220, 80, 80));
    v::video_puts((bx + 3) / 8, (by + 1) / 16, b"X\0".as_ptr(), v::COLOR_WHITE, v::rgb(180, 40, 40));
}

// --------------------------------------------------------------------------
// Строковые утилиты
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_str_concat(dst: *mut u8, src: *const u8) {
    strcat(dst, src);
}

unsafe extern "C" fn ap_str_ncompare(a: *const u8, b: *const u8, n: i32) -> i32 {
    strncmp(a, b, n as usize)
}

unsafe extern "C" fn ap_str_to_int(s: *const u8) -> i32 {
    atoi(s)
}

unsafe extern "C" fn ap_int_to_str(val: i32, buf: *mut u8) {
    itoa(val, buf, 10);
}

// --------------------------------------------------------------------------
// Мышь
// --------------------------------------------------------------------------

unsafe extern "C" fn ap_mouse_get_state(out: *mut ApiMouseState) {
    if out.is_null() {
        return;
    }
    let ms = mouse_get_state();
    let val = ApiMouseState {
        x: ms.x,
        y: ms.y,
        buttons: ms.buttons,
        wheel: ms.wheel,
    };
    ptr::write_unaligned(out, val);
}

unsafe extern "C" fn ap_mouse_set_pos(x: i32, y: i32) {
    mouse_set_pos(x, y);
}

unsafe extern "C" fn ap_mouse_set_sens(sx: i32, sy: i32) {
    mouse_set_sensitivity(sx, sy);
}

// --------------------------------------------------------------------------
// Установка таблицы
// --------------------------------------------------------------------------

/// Заполнение таблицы API по адресу 0x100000 (`api_setup_table`).
pub fn api_setup_table() {
    let t = API_TABLE_ADDRESS as *mut ApiTable;
    unsafe {
        ptr::write(t, core::mem::zeroed());
        (*t).print = Some(ap_print);
        (*t).println = Some(ap_println);
        (*t).print_char = Some(ap_print_char);
        (*t).print_int = Some(ap_print_int);
        (*t).print_hex = Some(ap_print_hex);
        (*t).set_color = Some(ap_set_color);

        (*t).read_char = Some(ap_read_char);
        (*t).read_line = Some(ap_read_line);

        (*t).clear_screen = Some(ap_clear);
        (*t).set_cursor = Some(ap_set_cur);
        (*t).get_cursor = Some(ap_get_cur);

        (*t).draw_box = Some(ap_draw_box);
        (*t).draw_box_titled = Some(ap_draw_box_titled);
        (*t).fill_rect = Some(ap_fill_rect);
        (*t).draw_hline = Some(ap_draw_hline);
        (*t).draw_vline = Some(ap_draw_vline);

        (*t).draw_window = Some(ap_draw_window);
        (*t).draw_button = Some(ap_draw_button);
        (*t).draw_input = Some(ap_draw_input);
        (*t).draw_progress = Some(ap_draw_progress);

        (*t).str_compare = Some(ap_strcmp);
        (*t).str_length = Some(ap_strlen);
        (*t).str_copy = Some(ap_strcpy);
        (*t).mem_copy = Some(ap_memcpy);
        (*t).mem_set = Some(ap_memset);

        (*t).get_time = Some(ap_get_time);
        (*t).get_date = Some(ap_get_date);
        (*t).delay = Some(ap_delay);

        (*t).file_exists = Some(ap_fexists);
        (*t).file_read = Some(ap_fread);
        (*t).file_size = Some(ap_fsize);
        (*t).file_write = Some(ap_fwrite);

        (*t).has_key = Some(ap_haskey);
        (*t).get_key = Some(ap_getkey);
        (*t).clear_keys = Some(ap_clearkeys);

        (*t).get_sysinfo = Some(ap_get_sysinfo);
        (*t).set_username = Some(ap_set_username);
        (*t).set_hostname = Some(ap_set_hostname);

        (*t).gfx_pixel = Some(ap_gfx_pixel);
        (*t).gfx_line = Some(ap_gfx_line);
        (*t).gfx_rect = Some(ap_gfx_rect);
        (*t).gfx_fillrect = Some(ap_gfx_fillrect);
        (*t).gfx_circle = Some(ap_gfx_circle);
        (*t).gfx_fillcircle = Some(ap_gfx_fillcircle);
        (*t).gfx_char = Some(ap_gfx_char);
        (*t).gfx_text = Some(ap_gfx_text);
        (*t).gfx_width = Some(ap_gfx_width);
        (*t).gfx_height = Some(ap_gfx_height);
        (*t).gfx_blit = Some(ap_gfx_blit);

        (*t).gui_label = Some(ap_gui_label);
        (*t).gui_checkbox = Some(ap_gui_checkbox);
        (*t).gui_menubar = Some(ap_gui_menubar);
        (*t).gui_tabbar = Some(ap_gui_tabbar);
        (*t).gui_draw_cursor = Some(ap_gui_draw_cursor);
        (*t).gui_listbox = Some(ap_gui_listbox);
        (*t).gui_scrollbar = Some(ap_gui_scrollbar);
        (*t).gui_popup_menu = Some(ap_gui_popup_menu);
        (*t).gui_titlebar = Some(ap_gui_titlebar);

        (*t).str_concat = Some(ap_str_concat);
        (*t).str_ncompare = Some(ap_str_ncompare);
        (*t).str_to_int = Some(ap_str_to_int);
        (*t).int_to_str = Some(ap_int_to_str);

        (*t).mouse_get_state = Some(ap_mouse_get_state);
        (*t).mouse_set_pos = Some(ap_mouse_set_pos);
        (*t).mouse_set_sens = Some(ap_mouse_set_sens);
    }
}

/// Инициализация API (`api_init`).
pub fn api_init() {
    api_setup_table();
}