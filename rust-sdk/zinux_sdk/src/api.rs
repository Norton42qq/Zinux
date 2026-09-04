pub mod ids {
    pub const PRINT: u32 = 1;
    pub const PRINTLN: u32 = 2;
    pub const PRINT_CHAR: u32 = 3;
    pub const PRINT_INT: u32 = 4;
    pub const PRINT_HEX: u32 = 5;
    pub const SET_COLOR: u32 = 6;
    pub const READ_CHAR: u32 = 7;
    pub const READ_LINE: u32 = 8;
    pub const CLEAR_SCREEN: u32 = 9;
    pub const SET_CURSOR: u32 = 10;
    pub const GET_CURSOR: u32 = 11;
    pub const DRAW_BOX: u32 = 12;
    pub const DRAW_BOX_TITLED: u32 = 13;
    pub const FILL_RECT: u32 = 14;
    pub const DRAW_HLINE: u32 = 15;
    pub const DRAW_VLINE: u32 = 16;
    pub const DRAW_WINDOW: u32 = 17;
    pub const DRAW_BUTTON: u32 = 18;
    pub const DRAW_INPUT: u32 = 19;
    pub const DRAW_PROGRESS: u32 = 20;
    pub const STR_COMPARE: u32 = 21;
    pub const STR_LENGTH: u32 = 22;
    pub const STR_COPY: u32 = 23;
    pub const MEM_COPY: u32 = 24;
    pub const MEM_SET: u32 = 25;
    pub const GET_TIME: u32 = 26;
    pub const GET_DATE: u32 = 27;
    pub const DELAY: u32 = 28;
    pub const FILE_EXISTS: u32 = 29;
    pub const FILE_READ: u32 = 30;
    pub const FILE_SIZE: u32 = 31;
    pub const FILE_WRITE: u32 = 32;
    pub const HAS_KEY: u32 = 33;
    pub const GET_KEY: u32 = 34;
    pub const CLEAR_KEYS: u32 = 35;
    pub const GET_SYSINFO: u32 = 36;
    pub const SET_USERNAME: u32 = 37;
    pub const SET_HOSTNAME: u32 = 38;
    pub const GFX_PIXEL: u32 = 39;
    pub const GFX_LINE: u32 = 40;
    pub const GFX_RECT: u32 = 41;
    pub const GFX_FILLRECT: u32 = 42;
    pub const GFX_CIRCLE: u32 = 43;
    pub const GFX_FILLCIRCLE: u32 = 44;
    pub const GFX_CHAR: u32 = 45;
    pub const GFX_TEXT: u32 = 46;
    pub const GFX_WIDTH: u32 = 47;
    pub const GFX_HEIGHT: u32 = 48;
    pub const GFX_BLIT: u32 = 49;
    pub const GUI_LABEL: u32 = 50;
    pub const GUI_CHECKBOX: u32 = 51;
    pub const GUI_MENUBAR: u32 = 52;
    pub const GUI_TABBAR: u32 = 53;
    pub const GUI_DRAW_CURSOR: u32 = 54;
    pub const GUI_LISTBOX: u32 = 55;
    pub const GUI_SCROLLBAR: u32 = 56;
    pub const GUI_POPUP_MENU: u32 = 57;
    pub const GUI_TITLEBAR: u32 = 58;
    pub const STR_CONCAT: u32 = 59;
    pub const STR_NCOMPARE: u32 = 60;
    pub const STR_TO_INT: u32 = 61;
    pub const INT_TO_STR: u32 = 62;
    pub const MOUSE_GET_STATE: u32 = 63;
    pub const MOUSE_SET_POS: u32 = 64;
    pub const MOUSE_SET_SENS: u32 = 65;
}

pub mod keys {
    pub const UP: u8 = 128;
    pub const DOWN: u8 = 129;
    pub const LEFT: u8 = 130;
    pub const RIGHT: u8 = 131;
    pub const ESC: u8 = 27;
    pub const ENTER: u8 = b'\n';
    pub const SPACE: u8 = b' ';
    pub const BACKSPACE: u8 = 8;
    pub const F1: u8 = 132;
    pub const F2: u8 = 133;
    pub const F3: u8 = 134;
    pub const F4: u8 = 135;
}

pub mod colors {
    pub const BLACK: u32 = 0x0000_0000;
    pub const BLUE: u32 = 0x0000_00AA;
    pub const GREEN: u32 = 0x0000_AA00;
    pub const CYAN: u32 = 0x0000_AAAA;
    pub const RED: u32 = 0x00AA_0000;
    pub const MAGENTA: u32 = 0x00AA_00AA;
    pub const BROWN: u32 = 0x00AA_5500;
    pub const LIGHT_GREY: u32 = 0x00AA_AAAA;
    pub const DARK_GREY: u32 = 0x0055_5555;
    pub const LIGHT_BLUE: u32 = 0x0055_55FF;
    pub const LIGHT_GREEN: u32 = 0x0055_FF55;
    pub const LIGHT_CYAN: u32 = 0x0055_FFFF;
    pub const LIGHT_RED: u32 = 0x00FF_5555;
    pub const LIGHT_MAGENTA: u32 = 0x00FF_55FF;
    pub const YELLOW: u32 = 0x00FF_FF55;
    pub const WHITE: u32 = 0x00FF_FFFF;
    #[inline(always)]
    pub const fn rgb(r: u32, g: u32, b: u32) -> u32 { (r << 16) | (g << 8) | b }
}

pub mod pseudo {
    pub const HLINE: u8 = 0xC4;
    pub const VLINE: u8 = 0xB3;
    pub const TL: u8 = 0xDA;
    pub const TR: u8 = 0xBF;
    pub const BL: u8 = 0xC0;
    pub const BR: u8 = 0xD9;
    pub const SHADE: u8 = 0xB0;
    pub const BLOCK: u8 = 0xDB;
}

use crate::syscall;

fn call(id: u32, args: &[u32]) -> u32 {
    syscall::api_call(id, args)
}

fn copy_str_to_buf(buf: &mut [u8], s: &str) {
    let bytes = s.as_bytes();
    let n = bytes.len().min(buf.len() - 1);
    buf[..n].copy_from_slice(&bytes[..n]);
    buf[n] = 0;
}

fn str_buf(s: &str) -> [u8; 1024] {
    let mut buf = [0u8; 1024];
    let bytes = s.as_bytes();
    let n = bytes.len().min(1023);
    buf[..n].copy_from_slice(&bytes[..n]);
    buf[n] = 0;
    buf
}

fn small_buf(s: &str) -> [u8; 256] {
    let mut buf = [0u8; 256];
    let bytes = s.as_bytes();
    let n = bytes.len().min(254);
    buf[..n].copy_from_slice(&bytes[..n]);
    buf[n] = 0;
    buf
}

pub fn print(s: &str) {
    let buf = str_buf(s);
    call(ids::PRINT, &[buf.as_ptr() as u32]);
}

pub fn println(s: &str) {
    let buf = str_buf(s);
    call(ids::PRINTLN, &[buf.as_ptr() as u32]);
}

pub fn putchar(c: u8) {
    call(ids::PRINT_CHAR, &[c as u32]);
}

pub fn print_int(v: i32) {
    call(ids::PRINT_INT, &[v as u32]);
}

pub fn print_hex(v: u32) {
    call(ids::PRINT_HEX, &[v]);
}

pub fn color(fg: u32, bg: u32) {
    call(ids::SET_COLOR, &[fg, bg]);
}

pub fn waitkey() -> u8 {
    loop {
        if has_key() != 0 {
            return get_key();
        }
    }
}

pub fn has_key() -> u32 {
    call(ids::HAS_KEY, &[])
}

pub fn get_key() -> u8 {
    call(ids::GET_KEY, &[]) as u8
}

pub fn clear_keys() {
    call(ids::CLEAR_KEYS, &[]);
}

pub fn clear() {
    call(ids::CLEAR_SCREEN, &[]);
}

pub fn set_cursor(x: i32, y: i32) {
    call(ids::SET_CURSOR, &[x as u32, y as u32]);
}

pub fn get_cursor() -> (i32, i32) {
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    call(ids::GET_CURSOR, &[&mut x as *mut i32 as u32, &mut y as *mut i32 as u32]);
    (x, y)
}

pub fn read_char() -> u8 {
    call(ids::READ_CHAR, &[]) as u8
}

pub fn read_line(buf: &mut [u8], max_len: i32) {
    call(ids::READ_LINE, &[buf.as_mut_ptr() as u32, max_len as u32]);
}

pub fn delay(ms: u32) {
    call(ids::DELAY, &[ms]);
}

pub fn draw_box(x: i32, y: i32, w: i32, h: i32, fg: u32, bg: u32) {
    call(ids::DRAW_BOX, &[x as u32, y as u32, w as u32, h as u32, fg, bg]);
}

pub fn draw_box_titled(x: i32, y: i32, w: i32, h: i32, title: &str, fg: u32, bg: u32) {
    let buf = small_buf(title);
    call(ids::DRAW_BOX_TITLED, &[x as u32, y as u32, w as u32, h as u32, buf.as_ptr() as u32, fg, bg]);
}

pub fn fill_rect(x: i32, y: i32, w: i32, h: i32, ch: u8, fg: u32, bg: u32) {
    call(ids::FILL_RECT, &[x as u32, y as u32, w as u32, h as u32, ch as u32, fg, bg]);
}

pub fn hline(x: i32, y: i32, len: i32, fg: u32, bg: u32) {
    call(ids::DRAW_HLINE, &[x as u32, y as u32, len as u32, fg, bg]);
}

pub fn vline(x: i32, y: i32, len: i32, fg: u32, bg: u32) {
    call(ids::DRAW_VLINE, &[x as u32, y as u32, len as u32, fg, bg]);
}

pub fn draw_window(x: i32, y: i32, w: i32, h: i32, title: &str) {
    let buf = small_buf(title);
    call(ids::DRAW_WINDOW, &[x as u32, y as u32, w as u32, h as u32, buf.as_ptr() as u32]);
}

pub fn draw_button(x: i32, y: i32, text: &str, selected: bool) {
    let buf = small_buf(text);
    call(ids::DRAW_BUTTON, &[x as u32, y as u32, buf.as_ptr() as u32, if selected { 1 } else { 0 }]);
}

pub fn textbox(x: i32, y: i32, w: i32, text: &str, active: bool) {
    let buf = small_buf(text);
    call(ids::DRAW_INPUT, &[x as u32, y as u32, w as u32, buf.as_ptr() as u32, if active { 1 } else { 0 }]);
}

pub fn progress(x: i32, y: i32, w: i32, percent: i32) {
    call(ids::DRAW_PROGRESS, &[x as u32, y as u32, w as u32, percent as u32]);
}

pub fn str_compare(a: &str, b: &str) -> i32 {
    let ab = small_buf(a);
    let bb = small_buf(b);
    call(ids::STR_COMPARE, &[ab.as_ptr() as u32, bb.as_ptr() as u32]) as i32
}

pub fn streq(a: &str, b: &str) -> bool {
    str_compare(a, b) == 0
}

pub fn slen(s: &str) -> i32 {
    s.as_bytes().len() as i32
}

pub fn str_ncompare(a: &str, b: &str, n: i32) -> i32 {
    let ab = small_buf(a);
    let bb = small_buf(b);
    call(ids::STR_NCOMPARE, &[ab.as_ptr() as u32, bb.as_ptr() as u32, n as u32]) as i32
}

pub fn str_to_int(s: &str) -> i32 {
    let buf = small_buf(s);
    call(ids::STR_TO_INT, &[buf.as_ptr() as u32]) as i32
}

pub fn int_to_str(v: i32, buf: &mut [u8]) {
    call(ids::INT_TO_STR, &[v as u32, buf.as_mut_ptr() as u32]);
}

pub fn get_time() -> (u8, u8, u8) {
    let mut h: u8 = 0;
    let mut m: u8 = 0;
    let mut s: u8 = 0;
    call(ids::GET_TIME, &[&mut h as *mut u8 as u32, &mut m as *mut u8 as u32, &mut s as *mut u8 as u32]);
    (h, m, s)
}

pub fn get_date() -> (u8, u8, u16) {
    let mut d: u8 = 0;
    let mut mo: u8 = 0;
    let mut y: u16 = 0;
    call(ids::GET_DATE, &[&mut d as *mut u8 as u32, &mut mo as *mut u8 as u32, &mut y as *mut u16 as u32]);
    (d, mo, y)
}

#[repr(C, packed)]
pub struct SysInfo {
    pub username: [u8; 32],
    pub hostname: [u8; 32],
    pub os_name: [u8; 32],
    pub os_version: [u8; 16],
    pub cwd: [u8; 64],
    pub mem_kb: u32,
}

pub fn get_sysinfo() -> SysInfo {
    let mut info = SysInfo {
        username: [0u8; 32],
        hostname: [0u8; 32],
        os_name: [0u8; 32],
        os_version: [0u8; 16],
        cwd: [0u8; 64],
        mem_kb: 0,
    };
    call(ids::GET_SYSINFO, &[&mut info as *mut SysInfo as u32]);
    info
}

pub fn cstr(buf: &[u8]) -> &str {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    core::str::from_utf8(&buf[..end]).unwrap_or("")
}

pub fn gfx_pixel(x: i32, y: i32, c: u32) {
    call(ids::GFX_PIXEL, &[x as u32, y as u32, c]);
}

pub fn gfx_line(x0: i32, y0: i32, x1: i32, y1: i32, c: u32) {
    call(ids::GFX_LINE, &[x0 as u32, y0 as u32, x1 as u32, y1 as u32, c]);
}

pub fn gfx_rect(x: i32, y: i32, w: i32, h: i32, c: u32) {
    call(ids::GFX_RECT, &[x as u32, y as u32, w as u32, h as u32, c]);
}

pub fn gfx_fillrect(x: i32, y: i32, w: i32, h: i32, c: u32) {
    call(ids::GFX_FILLRECT, &[x as u32, y as u32, w as u32, h as u32, c]);
}

pub fn gfx_circle(cx: i32, cy: i32, r: i32, c: u32) {
    call(ids::GFX_CIRCLE, &[cx as u32, cy as u32, r as u32, c]);
}

pub fn gfx_fillcircle(cx: i32, cy: i32, r: i32, c: u32) {
    call(ids::GFX_FILLCIRCLE, &[cx as u32, cy as u32, r as u32, c]);
}

pub fn gfx_char(x: i32, y: i32, c: u8, fg: u32, bg: u32) {
    call(ids::GFX_CHAR, &[x as u32, y as u32, c as u32, fg, bg]);
}

pub fn gfx_text(x: i32, y: i32, s: &str, fg: u32, bg: u32) {
    let buf = str_buf(s);
    call(ids::GFX_TEXT, &[x as u32, y as u32, buf.as_ptr() as u32, fg, bg]);
}

pub fn screen_w() -> i32 {
    call(ids::GFX_WIDTH, &[]) as i32
}

pub fn screen_h() -> i32 {
    call(ids::GFX_HEIGHT, &[]) as i32
}

pub fn gfx_blit(x: i32, y: i32, w: i32, h: i32, pixels: &[u32]) {
    call(ids::GFX_BLIT, &[x as u32, y as u32, w as u32, h as u32, pixels.as_ptr() as u32]);
}

pub fn gui_label(x: i32, y: i32, text: &str, color: u32) {
    let buf = small_buf(text);
    call(ids::GUI_LABEL, &[x as u32, y as u32, buf.as_ptr() as u32, color]);
}

pub fn gui_checkbox(x: i32, y: i32, label: &str, checked: bool) {
    let buf = small_buf(label);
    call(ids::GUI_CHECKBOX, &[x as u32, y as u32, buf.as_ptr() as u32, if checked { 1 } else { 0 }]);
}

pub fn gui_menubar(y: i32, items: &[&str], selected: i32) {
    let mut ptrs: [u32; 32] = [0; 32];
    let mut bufs: [[u8; 64]; 32] = [[0u8; 64]; 32];
    let n = items.len().min(32);
    for i in 0..n {
        copy_str_to_buf(&mut bufs[i], items[i]);
        ptrs[i] = bufs[i].as_ptr() as u32;
    }
    call(ids::GUI_MENUBAR, &[y as u32, ptrs.as_ptr() as u32, n as u32, selected as u32]);
}

pub fn gui_tabbar(x: i32, y: i32, w: i32, tabs: &[&str], active: i32) {
    let mut ptrs: [u32; 16] = [0; 16];
    let mut bufs: [[u8; 64]; 16] = [[0u8; 64]; 16];
    let n = tabs.len().min(16);
    for i in 0..n {
        copy_str_to_buf(&mut bufs[i], tabs[i]);
        ptrs[i] = bufs[i].as_ptr() as u32;
    }
    call(ids::GUI_TABBAR, &[x as u32, y as u32, w as u32, ptrs.as_ptr() as u32, n as u32, active as u32]);
}

pub fn gui_draw_cursor(x: i32, y: i32) {
    call(ids::GUI_DRAW_CURSOR, &[x as u32, y as u32]);
}

pub fn gui_listbox(x: i32, y: i32, w: i32, h: i32, items: &[&str], selected: i32) {
    let mut ptrs: [u32; 64] = [0; 64];
    let mut bufs: [[u8; 64]; 64] = [[0u8; 64]; 64];
    let n = items.len().min(64);
    for i in 0..n {
        copy_str_to_buf(&mut bufs[i], items[i]);
        ptrs[i] = bufs[i].as_ptr() as u32;
    }
    call(ids::GUI_LISTBOX, &[x as u32, y as u32, w as u32, h as u32, ptrs.as_ptr() as u32, n as u32, selected as u32]);
}

pub fn gui_scrollbar(x: i32, y: i32, h: i32, total: i32, visible: i32, pos: i32) {
    call(ids::GUI_SCROLLBAR, &[x as u32, y as u32, h as u32, total as u32, visible as u32, pos as u32]);
}

pub fn gui_popup_menu(x: i32, y: i32, items: &[&str], selected: i32) {
    let mut ptrs: [u32; 32] = [0; 32];
    let mut bufs: [[u8; 64]; 32] = [[0u8; 64]; 32];
    let n = items.len().min(32);
    for i in 0..n {
        copy_str_to_buf(&mut bufs[i], items[i]);
        ptrs[i] = bufs[i].as_ptr() as u32;
    }
    call(ids::GUI_POPUP_MENU, &[x as u32, y as u32, ptrs.as_ptr() as u32, n as u32, selected as u32]);
}

pub fn gui_titlebar(x: i32, y: i32, w: i32, title: &str, active: bool) {
    let buf = small_buf(title);
    call(ids::GUI_TITLEBAR, &[x as u32, y as u32, w as u32, buf.as_ptr() as u32, if active { 1 } else { 0 }]);
}

#[repr(C, packed)]
#[derive(Default, Copy, Clone)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub buttons: u8,
    pub wheel: i32,
}

pub fn mouse_get_state() -> MouseState {
    let mut ms = MouseState::default();
    call(ids::MOUSE_GET_STATE, &[&mut ms as *mut MouseState as u32]);
    ms
}

pub fn mouse_set_pos(x: i32, y: i32) {
    call(ids::MOUSE_SET_POS, &[x as u32, y as u32]);
}

pub fn mouse_set_sens(sx: i32, sy: i32) {
    call(ids::MOUSE_SET_SENS, &[sx as u32, sy as u32]);
}

pub fn mouse_left() -> bool {
    (mouse_get_state().buttons & 1) != 0
}

pub fn mouse_right() -> bool {
    (mouse_get_state().buttons & 2) != 0
}

pub fn file_exists(name: &str) -> bool {
    let buf = small_buf(name);
    call(ids::FILE_EXISTS, &[buf.as_ptr() as u32]) != 0
}

pub fn file_read(name: &str, buf: &mut [u8]) -> i32 {
    let nbuf = small_buf(name);
    call(ids::FILE_READ, &[nbuf.as_ptr() as u32, buf.as_mut_ptr() as u32, buf.len() as u32]) as i32
}

pub fn file_size(name: &str) -> u32 {
    let buf = small_buf(name);
    call(ids::FILE_SIZE, &[buf.as_ptr() as u32])
}

pub fn file_write(name: &str, data: &[u8]) -> i32 {
    let nbuf = small_buf(name);
    call(ids::FILE_WRITE, &[nbuf.as_ptr() as u32, data.as_ptr() as u32, data.len() as u32]) as i32
}

pub fn center_x(text: &str) -> i32 {
    (80 - slen(text)) / 2
}

pub fn msgbox(title: &str, message: &str) {
    let mut w = slen(message) + 4;
    if w < 30 { w = 30; }
    let h = 5;
    let x = (80 - w) / 2;
    let y = (30 - h) / 2;
    draw_window(x, y, w, h, title);
    set_cursor(x + 2, y + 2);
    color(colors::WHITE, colors::BLUE);
    print(message);
    draw_button(x + (w - 8) / 2, y + h - 2, "OK", true);
    waitkey();
    clear();
}
