#![no_std]
#![no_main]

use zinux_sdk::api;

fn clamp(v: i32, lo: i32, hi: i32) -> i32 {
    if v < lo { lo } else if v > hi { hi } else { v }
}

fn draw_brush(x: i32, y: i32, c: u32) {
    api::gfx_fillcircle(x, y, 2, c);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.zinux_entry")]
pub extern "C" fn _start() -> ! {
    unsafe { zinux_sdk::zero_bss(); }

    let sw = api::screen_w();
    let sh = api::screen_h();

    api::clear();
    api::gfx_text(8, 4, "Mouse Paint Test", api::colors::WHITE, api::colors::BLACK);
    api::gfx_text(8, 20, "LMB draw  RMB erase  C clear  ESC quit", api::colors::LIGHT_GREY, api::colors::BLACK);

    api::mouse_set_sens(1, 1);
    api::mouse_set_pos(sw / 2, sh / 2);

    let ms = api::mouse_get_state();
    let mut prev_x = clamp(ms.x, 0, sw - 7);
    let mut prev_y = clamp(ms.y, 0, sh - 12);
    api::gui_draw_cursor(prev_x, prev_y);

    loop {
        if api::has_key() != 0 {
            let k = api::get_key();
            if k == api::keys::ESC { break; }
            if k == b'c' || k == b'C' {
                api::clear();
                api::gfx_text(8, 4, "Mouse Paint Test", api::colors::WHITE, api::colors::BLACK);
                api::gfx_text(8, 20, "LMB draw  RMB erase  C clear  ESC quit", api::colors::LIGHT_GREY, api::colors::BLACK);
            }
        }

        let ms = api::mouse_get_state();
        let x = clamp(ms.x, 0, sw - 7);
        let y = clamp(ms.y, 0, sh - 12);

        if x != prev_x || y != prev_y {
            api::gui_draw_cursor(prev_x, prev_y);
            prev_x = x;
            prev_y = y;
            api::gui_draw_cursor(prev_x, prev_y);
        }

        if (ms.buttons & 1) != 0 {
            draw_brush(x, y, api::colors::WHITE);
            api::gui_draw_cursor(prev_x, prev_y);
            api::gui_draw_cursor(prev_x, prev_y);
        } else if (ms.buttons & 2) != 0 {
            draw_brush(x, y, api::colors::BLACK);
            api::gui_draw_cursor(prev_x, prev_y);
            api::gui_draw_cursor(prev_x, prev_y);
        }

        api::delay(8);
    }

    api::gui_draw_cursor(prev_x, prev_y);
    api::clear();
    zinux_sdk::syscall::syscall_exit(0)
}
