//! Порт `kernel/drivers/video.c` + `video.h` — текстовый/графический слой VBE.

use core::ptr;

use crate::arch::font::FONT_DATA;
use crate::arch::io::inb;
use zlibk::mem::memmove;

pub const VIDEO_INFO_ADDR: u32 = 0x8000;

pub const VIDEO_BPP_8: u32 = 8;
pub const VIDEO_BPP_15: u32 = 15;
pub const VIDEO_BPP_16: u32 = 16;
pub const VIDEO_BPP_24: u32 = 24;
pub const VIDEO_BPP_32: u32 = 32;

/// VBE-информация (20 байт, `VideoInfo` из video.h).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct VideoInfo {
    pub lfb_addr: u32,
    pub width: u16,
    pub height: u16,
    pub pitch: u32,
    pub bpp: u8,
    pub red_pos: u8,
    pub red_size: u8,
    pub green_pos: u8,
    pub green_size: u8,
    pub blue_pos: u8,
    pub blue_size: u8,
    pub reserved: u8,
}

/// Цвета (RGB-макросы из video.h).
#[inline]
pub fn rgb(r: u32, g: u32, b: u32) -> u32 {
    (r << 16) | (g << 8) | b
}

pub const COLOR_BLACK: u32 = 0x000000;
pub const COLOR_BLUE: u32 = 0x0000AA;
pub const COLOR_GREEN: u32 = 0x00AA00;
pub const COLOR_CYAN: u32 = 0x00AAAA;
pub const COLOR_RED: u32 = 0xAA0000;
pub const COLOR_MAGENTA: u32 = 0xAA00AA;
pub const COLOR_BROWN: u32 = 0xAA5500;
pub const COLOR_LIGHT_GREY: u32 = 0xAAAAAA;
pub const COLOR_DARK_GREY: u32 = 0x555555;
pub const COLOR_LIGHT_BLUE: u32 = 0x5555FF;
pub const COLOR_LIGHT_GREEN: u32 = 0x55FF55;
pub const COLOR_LIGHT_CYAN: u32 = 0x55FFFF;
pub const COLOR_LIGHT_RED: u32 = 0xFF5555;
pub const COLOR_LIGHT_MAGENTA: u32 = 0xFF55FF;
pub const COLOR_YELLOW: u32 = 0xFFFF55;
pub const COLOR_WHITE: u32 = 0xFFFFFF;

// Состояние драйвера
static mut G_VI: VideoInfo = VideoInfo {
    lfb_addr: 0,
    width: 0,
    height: 0,
    pitch: 0,
    bpp: 0,
    red_pos: 0,
    red_size: 0,
    green_pos: 0,
    green_size: 0,
    blue_pos: 0,
    blue_size: 0,
    reserved: 0,
};
static mut G_FB: *mut u8 = ptr::null_mut();
static mut G_FG: u32 = COLOR_WHITE;
static mut G_BG: u32 = COLOR_BLACK;
static mut G_CUR_X: i32 = 0;
static mut G_CUR_Y: i32 = 0;
static mut G_BYTES_PP: i32 = 4;

// Текстовый размер экрана
pub static mut VIDEO_TEXT_COLS: i32 = 80;
pub static mut VIDEO_TEXT_ROWS: i32 = 25;

/// Взять копию VideoInfo (безопасное чтение из packed-статика).
fn vi() -> VideoInfo {
    unsafe { G_VI }
}

// UTF-8 -> CP866 (Для отображения русских букв без escape-последовательностей)
fn utf8_to_cp866(s: *const u8, adv: *mut i32) -> u8 {
    unsafe {
        let b0 = *s.add(0);
        *adv = 1;

        if b0 < 0x80 {
            return b0;
        }

        if b0 == 0xD0 {
            let b1 = *s.add(1);
            if b1 == 0 {
                return b'?';
            }
            *adv = 2;
            if b1 == 0x81 {
                return 0xF0; // Ё
            }
            if b1 >= 0x90 && b1 <= 0xAF {
                return (0x80 + (b1 - 0x90)) as u8; // А..Я
            }
            if b1 >= 0xB0 && b1 <= 0xBF {
                return (0xA0 + (b1 - 0xB0)) as u8; // а..п
            }
            return b'?';
        }

        if b0 == 0xD1 {
            let b1 = *s.add(1);
            if b1 == 0 {
                return b'?';
            }
            *adv = 2;
            if b1 == 0x91 {
                return 0xF1; // ё
            }
            if b1 >= 0x80 && b1 <= 0x8F {
                return (0xE0 + (b1 - 0x80)) as u8; // р..я
            }
            return b'?';
        }

        b'?'
    }
}

// Упаковка цвета под нативный формат
fn color_pack(rgb: u32) -> u32 {
    let v = vi();
    let r = ((rgb >> 16) & 0xFF) as u32;
    let g = ((rgb >> 8) & 0xFF) as u32;
    let b = (rgb & 0xFF) as u32;
    match v.bpp {
        32 | 24 => {
            (r << v.red_pos) | (g << v.green_pos) | (b << v.blue_pos)
        }
        16 => {
            let rv = (r >> (8 - v.red_size)) & ((1u32 << v.red_size) - 1);
            let gv = (g >> (8 - v.green_size)) & ((1u32 << v.green_size) - 1);
            let bv = (b >> (8 - v.blue_size)) & ((1u32 << v.blue_size) - 1);
            (rv << v.red_pos) | (gv << v.green_pos) | (bv << v.blue_pos)
        }
        15 => {
            let rv = (r >> 3) & 0x1F;
            let gv = (g >> 3) & 0x1F;
            let bv = (b >> 3) & 0x1F;
            (rv << 10) | (gv << 5) | bv
        }
        8 => ((r >> 5) << 5) | ((g >> 5) << 2) | (b >> 6),
        _ => ((r >> 5) << 5) | ((g >> 5) << 2) | (b >> 6),
    }
}

// Запись пикселя в framebuffer
fn fb_write_pixel(x: i32, y: i32, native: u32) {
    let v = vi();
    let bytes_pp = unsafe { G_BYTES_PP } as usize;
    unsafe {
        let ptr: *mut u8 = G_FB
            .add((y as u32).wrapping_mul(v.pitch).wrapping_add((x as u32).wrapping_mul(bytes_pp as u32)) as usize);
        match v.bpp {
            32 => ptr::write_volatile(ptr as *mut u32, native),
            24 => {
                ptr::write_volatile(ptr.add(0), (native & 0xFF) as u8);
                ptr::write_volatile(ptr.add(1), ((native >> 8) & 0xFF) as u8);
                ptr::write_volatile(ptr.add(2), ((native >> 16) & 0xFF) as u8);
            }
            15 | 16 => ptr::write_volatile(ptr as *mut u16, native as u16),
            _ => ptr::write_volatile(ptr, native as u8),
        }
    }
}

// Чтение пикселя из framebuffer
fn fb_read_pixel(x: i32, y: i32) -> u32 {
    let v = vi();
    let bytes_pp = unsafe { G_BYTES_PP } as usize;
    unsafe {
        let ptr: *const u8 = G_FB
            .add((y as u32).wrapping_mul(v.pitch).wrapping_add((x as u32).wrapping_mul(bytes_pp as u32)) as usize);
        match v.bpp {
            32 => ptr::read_volatile(ptr as *const u32),
            24 => {
                (ptr::read_volatile(ptr.add(0)) as u32)
                    | ((ptr::read_volatile(ptr.add(1)) as u32) << 8)
                    | ((ptr::read_volatile(ptr.add(2)) as u32) << 16)
            }
            15 | 16 => ptr::read_volatile(ptr as *const u16) as u32,
            8 => ptr::read_volatile(ptr) as u32,
            _ => 0,
        }
    }
}

/// Инициализация видеорежима (порт `video_init`).
pub fn video_init() -> i32 {
    unsafe {
        // Загрузчик кладёт VideoInfo по адресу VIDEO_INFO_ADDR
        let src = VIDEO_INFO_ADDR as *const VideoInfo;
        let s = *src;

        if s.lfb_addr == 0 || s.lfb_addr == 0xFFFFFFFF {
            return -1;
        }
        if s.width == 0 || s.height == 0 {
            return -1;
        }
        let bp = s.bpp;
        if bp != 8 && bp != 15 && bp != 16 && bp != 24 && bp != 32 {
            return -1;
        }

        G_VI = s;
        G_FB = s.lfb_addr as *mut u8;

        // Байт на пиксель
        G_BYTES_PP = (bp as i32 + 7) / 8;

        // Заполнить поля маски для 8bpp если загрузчик не заполнил их
        if G_VI.bpp == 8 {
            G_VI.red_pos = 5;
            G_VI.red_size = 3;
            G_VI.green_pos = 2;
            G_VI.green_size = 3;
            G_VI.blue_pos = 0;
            G_VI.blue_size = 2;
        }

        VIDEO_TEXT_COLS = (G_VI.width as i32) / 8;
        VIDEO_TEXT_ROWS = (G_VI.height as i32) / 16;

        video_clear(COLOR_BLACK);
        0
    }
}

/// Получение VBE-информации (порт `video_get_info`).
pub fn video_get_info() -> *const VideoInfo {
    unsafe { &raw const G_VI }
}

// Пиксели
pub fn video_pixel(x: i32, y: i32, color: u32) {
    let v = vi();
    if (x as u32) >= v.width as u32 || (y as u32) >= v.height as u32 {
        return;
    }
    fb_write_pixel(x, y, color_pack(color));
}

pub fn video_getpixel(x: i32, y: i32) -> u32 {
    let v = vi();
    if (x as u32) >= v.width as u32 || (y as u32) >= v.height as u32 {
        return 0;
    }
    fb_read_pixel(x, y)
}

pub fn video_clear(color: u32) {
    let v = vi();
    let native = color_pack(color);
    unsafe {
        if v.bpp == 32 {
            // Быстрый путь 32bpp
            for y in 0..v.height {
                let row: *mut u32 = G_FB.add(((y as u32) * v.pitch) as usize) as *mut u32;
                for x in 0..v.width {
                    ptr::write_volatile(row.add(x as usize), native);
                }
            }
        } else if v.bpp == 16 || v.bpp == 15 {
            // Быстрый путь 16/15bpp
            for y in 0..v.height {
                let row: *mut u16 = G_FB.add(((y as u32) * v.pitch) as usize) as *mut u16;
                for x in 0..v.width {
                    ptr::write_volatile(row.add(x as usize), native as u16);
                }
            }
        } else {
            // Универсальный путь для 8/24bpp
            for y in 0..v.height {
                for x in 0..v.width {
                    fb_write_pixel(x as i32, y as i32, native);
                }
            }
        }
    }
}

// Примитивы
pub fn video_hline(x: i32, y: i32, len: i32, color: u32) {
    let v = vi();
    if (y as u32) >= v.height as u32 {
        return;
    }
    let native = color_pack(color);
    let mut x0 = x;
    let mut x2 = x + len;
    if x0 < 0 {
        x0 = 0;
    }
    if x2 > v.width as i32 {
        x2 = v.width as i32;
    }
    if x0 >= x2 {
        return;
    }
    unsafe {
        if v.bpp == 32 {
            let row: *mut u32 = G_FB.add(((y as u32) * v.pitch) as usize) as *mut u32;
            for i in x0..x2 {
                ptr::write_volatile(row.add(i as usize), native);
            }
        } else if v.bpp == 16 || v.bpp == 15 {
            let row: *mut u16 = G_FB.add(((y as u32) * v.pitch) as usize) as *mut u16;
            for i in x0..x2 {
                ptr::write_volatile(row.add(i as usize), native as u16);
            }
        } else {
            for i in x0..x2 {
                fb_write_pixel(i, y, native);
            }
        }
    }
}

pub fn video_vline(x: i32, y: i32, len: i32, color: u32) {
    let v = vi();
    if (x as u32) >= v.width as u32 {
        return;
    }
    let native = color_pack(color);
    let mut y0 = y;
    let mut y2 = y + len;
    if y0 < 0 {
        y0 = 0;
    }
    if y2 > v.height as i32 {
        y2 = v.height as i32;
    }
    for i in y0..y2 {
        fb_write_pixel(x, i, native);
    }
}

pub fn video_line(mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u32) {
    let mut dx = x1 - x0;
    if dx < 0 {
        dx = -dx;
    }
    let mut dy = y1 - y0;
    if dy < 0 {
        dy = -dy;
    }
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    loop {
        video_pixel(x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x0 += sx;
        }
        if e2 < dx {
            err += dx;
            y0 += sy;
        }
    }
}

pub fn video_rect(x: i32, y: i32, w: i32, h: i32, color: u32) {
    video_hline(x, y, w, color);
    video_hline(x, y + h - 1, w, color);
    video_vline(x, y, h, color);
    video_vline(x + w - 1, y, h, color);
}

pub fn video_fillrect(x: i32, y: i32, w: i32, h: i32, color: u32) {
    for i in 0..h {
        video_hline(x, y + i, w, color);
    }
}

pub fn video_circle(cx: i32, cy: i32, r: i32, color: u32) {
    let mut px = 0;
    let mut py = r;
    let mut d = 3 - 2 * r;
    while py >= px {
        video_pixel(cx + px, cy + py, color);
        video_pixel(cx - px, cy + py, color);
        video_pixel(cx + px, cy - py, color);
        video_pixel(cx - px, cy - py, color);
        video_pixel(cx + py, cy + px, color);
        video_pixel(cx - py, cy + px, color);
        video_pixel(cx + py, cy - px, color);
        video_pixel(cx - py, cy - px, color);
        px += 1;
        // d = (d > 0) ? (d + 4*(px - py--) + 10) : (d + 4*px + 6);
        if d > 0 {
            py -= 1;
            d = d + 4 * (px - py) + 10;
        } else {
            d = d + 4 * px + 6;
        }
    }
}

pub fn video_fillcircle(cx: i32, cy: i32, r: i32, color: u32) {
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy <= r * r {
                video_pixel(cx + dx, cy + dy, color);
            }
        }
    }
}

// Текст
pub fn video_putchar(col: i32, row: i32, c: u8, fg: u32, bg: u32) {
    let v = vi();
    let nat_fg = color_pack(fg);
    let nat_bg = color_pack(bg);
    let glyph = &FONT_DATA[c as usize];
    let px0 = col * 8;
    let py0 = row * 16;
    for r in 0..16u8 {
        let bits = glyph[r as usize];
        for b in 0..8u8 {
            let px = px0 + b as i32;
            let py = py0 + r as i32;
            if (px as u32) >= v.width as u32 || (py as u32) >= v.height as u32 {
                continue;
            }
            let colr = if bits & (0x80 >> b) != 0 { nat_fg } else { nat_bg };
            fb_write_pixel(px, py, colr);
        }
    }
}

pub fn video_puts(mut col: i32, row: i32, str_: *const u8, fg: u32, bg: u32) {
    let mut s = str_;
    unsafe {
        while *s != 0 {
            let mut adv: i32 = 1;
            let ch = utf8_to_cp866(s, &mut adv);
            video_putchar(col, row, ch, fg, bg);
            col += 1;
            s = s.add(adv as usize);
        }
    }
}

// Внутренний скролл
fn scroll_up() {
    let v = vi();
    let line_bytes = (v.pitch as u32) * 16u32;
    let copy_size = (v.pitch as u32) * ((v.height as u32) - 16u32);
    unsafe {
        memmove(G_FB, G_FB.add(line_bytes as usize), copy_size as usize);
    }
    let nat_bg = color_pack(unsafe { G_BG });
    for y in (v.height as i32) - 16 .. v.height as i32 {
        for x in 0..v.width as i32 {
            fb_write_pixel(x, y, nat_bg);
        }
    }
}

pub fn video_print_char(c: u8) {
    if c == b'\n' {
        unsafe {
            G_CUR_X = 0;
            G_CUR_Y += 1;
            if G_CUR_Y >= VIDEO_TEXT_ROWS {
                scroll_up();
                G_CUR_Y = VIDEO_TEXT_ROWS - 1;
            }
        }
    } else if c == 0x08 {
        unsafe {
            if G_CUR_X > 0 {
                G_CUR_X -= 1;
                video_putchar(G_CUR_X, G_CUR_Y, b' ', G_FG, G_BG);
            }
        }
    } else if c == b'\r' {
        unsafe { G_CUR_X = 0; }
    } else if c >= 32 {
        unsafe {
            video_putchar(G_CUR_X, G_CUR_Y, c, G_FG, G_BG);
            G_CUR_X += 1;
            if G_CUR_X >= VIDEO_TEXT_COLS {
                G_CUR_X = 0;
                G_CUR_Y += 1;
                if G_CUR_Y >= VIDEO_TEXT_ROWS {
                    scroll_up();
                    G_CUR_Y = VIDEO_TEXT_ROWS - 1;
                }
            }
        }
    }
}

pub fn video_print(str_: *const u8) {
    let mut s = str_;
    unsafe {
        while *s != 0 {
            let mut adv: i32 = 1;
            let ch = utf8_to_cp866(s, &mut adv);
            video_print_char(ch);
            s = s.add(adv as usize);
        }
    }
}

pub fn video_println(str_: *const u8) {
    video_print(str_);
    video_print_char(b'\n');
}

pub fn video_backspace() {
    video_print_char(0x08);
}

pub fn video_set_cursor(col: i32, row: i32) {
    unsafe {
        G_CUR_X = col;
        G_CUR_Y = row;
    }
}

pub fn video_get_cursor(col: *mut i32, row: *mut i32) {
    unsafe {
        if !col.is_null() {
            *col = G_CUR_X;
        }
        if !row.is_null() {
            *row = G_CUR_Y;
        }
    }
}

pub fn video_set_color(fg: u32, bg: u32) {
    unsafe {
        G_FG = fg;
        G_BG = bg;
    }
}

// Утилиты
pub fn video_vsync() {
    unsafe {
        while inb(0x3DA) & 8 != 0 {}
        while inb(0x3DA) & 8 == 0 {}
    }
}

pub fn video_width() -> i32 {
    vi().width as i32
}

pub fn video_height() -> i32 {
    vi().height as i32
}

pub fn video_put_dec(mut n: u32) {
    if n == 0 {
        video_print_char(b'0');
        return;
    }
    let mut buf = [0u8; 12];
    let mut i = 0usize;
    while n > 0 {
        buf[i] = b'0' + ((n % 10) as u8);
        i += 1;
        n /= 10;
    }
    while i > 0 {
        i -= 1;
        video_print_char(buf[i]);
    }
}

pub fn video_put_hex(n: u32) {
    const HEX: *const u8 = b"0123456789ABCDEF".as_ptr();
    video_print(b"0x\0".as_ptr());
    let mut i: i32 = 28;
    while i >= 0 {
        unsafe { video_print_char(*HEX.add(((n >> i) & 0xF) as usize)); }
        i -= 4;
    }
}