#![no_std]
#![no_main]

use zinux_sdk::api;

const ICON: [u32; 64] = [
    0,         api::colors::LIGHT_GREEN, 0,         api::colors::LIGHT_GREEN, 0,         api::colors::LIGHT_GREEN, 0,         0,
    api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::LIGHT_GREEN, 0,
    0,         api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::YELLOW, api::colors::YELLOW, api::colors::LIGHT_GREEN, 0,         0,
    api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::YELLOW, api::colors::WHITE, api::colors::YELLOW, api::colors::YELLOW, api::colors::LIGHT_GREEN, 0,
    0,         api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::YELLOW, api::colors::YELLOW, api::colors::LIGHT_GREEN, 0,         0,
    api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::LIGHT_GREEN, api::colors::YELLOW, api::colors::LIGHT_GREEN, 0,
    0,         api::colors::LIGHT_GREEN, 0,         api::colors::LIGHT_GREEN, 0,         api::colors::LIGHT_GREEN, 0,         0,
    0,         0,         0,         0,         0,         0,         0,         0,
];

fn draw_page_graphics() {
    api::clear();
    let sw = api::screen_w();
    let sh = api::screen_h();

    api::gfx_fillrect(0, 0, sw, 20, api::colors::rgb(20, 20, 60));
    api::gfx_text(8, 2, "ZINUX GUI DEMO - Pixel Graphics", api::colors::WHITE, api::colors::rgb(20, 20, 60));

    let palette: [u32; 15] = [
        api::colors::RED, api::colors::GREEN, api::colors::BLUE, api::colors::YELLOW, api::colors::MAGENTA, api::colors::CYAN,
        api::colors::LIGHT_RED, api::colors::LIGHT_GREEN, api::colors::LIGHT_BLUE, api::colors::LIGHT_CYAN,
        api::colors::LIGHT_MAGENTA, api::colors::YELLOW, api::colors::WHITE, api::colors::LIGHT_GREY, api::colors::DARK_GREY,
    ];
    for i in 0..15 {
        api::gfx_fillrect(30 + i * 50, 40, 44, 30, palette[i as usize]);
        api::gfx_rect(30 + i * 50, 40, 44, 30, api::colors::WHITE);
    }

    let cx = 100; let cy = 160;
    for a in 0..16i32 {
        let c = api::colors::rgb((a * 16) as u32, (255 - a * 12) as u32, 128);
        api::gfx_line(cx, cy, cx + a * 18, cy + 80, c);
    }

    for r in (5..=50).step_by(9) {
        let c = api::colors::rgb((r * 4) as u32, (200 - r * 2) as u32, (255 - r * 3) as u32);
        api::gfx_circle(500, 200, r, c);
    }
    api::gfx_fillcircle(650, 200, 35, api::colors::LIGHT_CYAN);
    api::gfx_fillcircle(650, 200, 20, api::colors::BLUE);
    api::gfx_fillcircle(650, 200, 8, api::colors::WHITE);

    for x in 0..200i32 {
        let c = api::colors::rgb((x * 255 / 200) as u32, 100, (255 - x * 255 / 200) as u32);
        api::gfx_line(800 + x, 40, 800 + x, 110, c);
    }
    api::gfx_rect(800, 40, 200, 70, api::colors::WHITE);
    api::gfx_text(800, 120, "gradient", api::colors::LIGHT_GREY, api::colors::BLACK);

    for row in 0..8i32 {
        for col in 0..8i32 {
            let px = ICON[(row * 8 + col) as usize];
            if px != 0 {
                api::gfx_fillrect(900 + col * 8, 150 + row * 8, 8, 8, px);
            }
        }
    }
    api::gfx_rect(900, 150, 64, 64, api::colors::DARK_GREY);
    api::gfx_text(896, 220, "blit 4x", api::colors::LIGHT_GREY, api::colors::BLACK);

    api::gfx_fillrect(0, sh - 24, sw, 24, api::colors::rgb(20, 20, 20));
    api::gfx_text(8, sh - 20, "[ 1 ] Graphics  [ 2 ] Widgets  [ 3 ] Colors  [ ESC ] Quit", api::colors::DARK_GREY, api::colors::rgb(20, 20, 20));
}

fn draw_page_widgets() {
    api::clear();
    let sw = api::screen_w();
    let sh = api::screen_h();

    api::gfx_fillrect(0, 0, sw, 20, api::colors::rgb(40, 20, 20));
    api::gfx_text(8, 2, "ZINUX GUI DEMO - Widgets", api::colors::WHITE, api::colors::rgb(40, 20, 20));

    api::draw_window(5, 2, 35, 16, " Settings ");
    api::gui_label(7, 4, "System options:", api::colors::LIGHT_CYAN);
    api::gui_checkbox(7, 6, "Enable sounds", true);
    api::gui_checkbox(7, 8, "Show clock", true);
    api::gui_checkbox(7, 10, "Dark mode", false);
    api::gui_checkbox(7, 12, "Auto-save", true);
    api::draw_button(7, 15, " Apply ", true);
    api::draw_button(18, 15, " Cancel ", false);

    api::draw_window(42, 2, 35, 16, " Progress ");
    api::gui_label(44, 4, "CPU:", api::colors::LIGHT_GREEN);
    api::set_cursor(50, 4); api::color(api::colors::LIGHT_GREEN, api::colors::BLACK); api::print("72%");
    api::progress(44, 5, 30, 72);
    api::gui_label(44, 7, "RAM:", api::colors::YELLOW);
    api::set_cursor(50, 7); api::color(api::colors::YELLOW, api::colors::BLACK); api::print("45%");
    api::progress(44, 8, 30, 45);
    api::gui_label(44, 10, "Disk:", api::colors::LIGHT_RED);
    api::set_cursor(50, 10); api::color(api::colors::LIGHT_RED, api::colors::BLACK); api::print("91%");
    api::progress(44, 11, 30, 91);

    let tabs: [&str; 4] = ["Files", "Network", "Display", "About"];
    api::gui_tabbar(5, 19, 60, &tabs, 2);
    api::fill_rect(5, 21, 60, 8, b' ', api::colors::WHITE, api::colors::rgb(10, 10, 40));
    api::draw_box(5, 21, 60, 8, api::colors::DARK_GREY, api::colors::rgb(10, 10, 40));
    api::gui_label(7, 22, "Display tab content:", api::colors::LIGHT_CYAN);
    api::gui_checkbox(7, 24, "1920x1080 @ 32bpp", true);
    api::gui_checkbox(7, 26, "VSync", false);

    let menu_items: [&str; 4] = ["File", "Edit", "View", "Help"];
    api::gui_menubar(sh / 16 - 3, &menu_items, 0);

    api::gfx_fillrect(0, sh - 24, sw, 24, api::colors::rgb(20, 20, 20));
    api::gfx_text(8, sh - 20, "[ 1 ] Graphics  [ 2 ] Widgets  [ 3 ] Colors  [ ESC ] Quit", api::colors::DARK_GREY, api::colors::rgb(20, 20, 20));
}

fn draw_page_colors() {
    api::clear();
    let sw = api::screen_w();
    let sh = api::screen_h();

    api::gfx_fillrect(0, 0, sw, 20, api::colors::rgb(20, 40, 20));
    api::gfx_text(8, 2, "ZINUX GUI DEMO - Color Palette", api::colors::WHITE, api::colors::rgb(20, 40, 20));

    let pal16: [u32; 16] = [
        api::colors::BLACK, api::colors::BLUE, api::colors::GREEN, api::colors::CYAN,
        api::colors::RED, api::colors::MAGENTA, api::colors::BROWN, api::colors::LIGHT_GREY,
        api::colors::DARK_GREY, api::colors::LIGHT_BLUE, api::colors::LIGHT_GREEN, api::colors::LIGHT_CYAN,
        api::colors::LIGHT_RED, api::colors::LIGHT_MAGENTA, api::colors::YELLOW, api::colors::WHITE,
    ];
    let names16: [&str; 16] = [
        "BLACK", "BLUE", "GREEN", "CYAN", "RED", "MAGENTA", "BROWN", "L.GREY",
        "D.GREY", "L.BLUE", "L.GREEN", "L.CYAN", "L.RED", "L.MAGENTA", "YELLOW", "WHITE",
    ];
    for i in 0..16 {
        let col = i % 8;
        let row = i / 8;
        api::gfx_fillrect(20 + col as i32 * 100, 35 + row as i32 * 50, 80, 30, pal16[i]);
        api::gfx_rect(20 + col as i32 * 100, 35 + row as i32 * 50, 80, 30, api::colors::DARK_GREY);
        api::gfx_text(20 + col as i32 * 100 + 4, 35 + row as i32 * 50 + 36, names16[i], api::colors::LIGHT_GREY, api::colors::BLACK);
    }

    api::gfx_text(20, 150, "RGB slices:", api::colors::LIGHT_CYAN, api::colors::BLACK);
    for g in 0..6i32 {
        for r in 0..32i32 {
            for b in 0..32i32 {
                let c = api::colors::rgb((r * 8) as u32, (g * 51) as u32, (b * 8) as u32);
                api::gfx_pixel(20 + g * 70 + r, 165 + b, c);
            }
        }
        let mut buf = [0u8; 4];
        buf[0] = b'G'; buf[1] = b'='; buf[2] = b'0' + g as u8; buf[3] = 0;
        api::gfx_text(20 + g * 70, 200, core::str::from_utf8(&buf).unwrap_or("G=?"), api::colors::LIGHT_GREY, api::colors::BLACK);
    }

    api::gfx_text(20, 220, "Gradients:", api::colors::LIGHT_CYAN, api::colors::BLACK);
    for x in 0..400i32 {
        api::gfx_line(20 + x, 235, 20 + x, 260, api::colors::rgb((x * 255 / 400) as u32, 0, 0));
        api::gfx_line(20 + x, 265, 20 + x, 290, api::colors::rgb(0, (x * 255 / 400) as u32, 0));
        api::gfx_line(20 + x, 295, 20 + x, 320, api::colors::rgb(0, 0, (x * 255 / 400) as u32));
        api::gfx_line(20 + x, 325, 20 + x, 350, api::colors::rgb((x * 255 / 400) as u32, (x * 255 / 400) as u32, 0));
    }

    api::gfx_fillrect(0, sh - 24, sw, 24, api::colors::rgb(20, 20, 20));
    api::gfx_text(8, sh - 20, "[ 1 ] Graphics  [ 2 ] Widgets  [ 3 ] Colors  [ ESC ] Quit", api::colors::DARK_GREY, api::colors::rgb(20, 20, 20));
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }

    draw_page_graphics();

    loop {
        let k = api::waitkey();
        if k == api::keys::ESC { break; }
        if k == b'1' { draw_page_graphics(); }
        if k == b'2' { draw_page_widgets(); }
        if k == b'3' { draw_page_colors(); }
    }

    api::clear();
    zinux_sdk::syscall::syscall_exit(0)
}
