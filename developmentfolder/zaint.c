#include "zxe_api.h"

static int clampi(int v, int lo, int hi) {
    if (v < lo) return lo;
    if (v > hi) return hi;
    return v;
}

static void draw_brush(int x, int y, uint32_t color) {
    int r = 2;
    fillcircle(x, y, r, color);
}

int main(int argc, char* argv[]) {
    (void)argc;
    (void)argv;

    int sw = screen_w();
    int sh = screen_h();

    clear();
    gfx_text(8, 4,  "Mouse Paint Test", WHITE, BLACK);
    gfx_text(8, 20, "LMB draw  RMB erase  C clear  ESC quit", LIGHT_GREY, BLACK);

    mouse_sens(1, 1);
    mouse_move(sw / 2, sh / 2);

    mouse_state_t ms;
    mouse_get(&ms);
    int prev_x = clampi(ms.x, 0, sw - 7);
    int prev_y = clampi(ms.y, 0, sh - 12);
    draw_cursor(prev_x, prev_y);

    while (1) {
        if (kbhit()) {
            char k = getch();
            if (k == KEY_ESC) break;
            if (k == 'c' || k == 'C') {
                clear();
                gfx_text(8, 4,  "Mouse Paint Test", WHITE, BLACK);
                gfx_text(8, 20, "LMB draw  RMB erase  C clear  ESC quit", LIGHT_GREY, BLACK);
            }
        }

        mouse_get(&ms);
        int x = clampi(ms.x, 0, sw - 7);
        int y = clampi(ms.y, 0, sh - 12);

        if (x != prev_x || y != prev_y) {
            draw_cursor(prev_x, prev_y);
            prev_x = x;
            prev_y = y;
            draw_cursor(prev_x, prev_y);
        }

        if (ms.buttons & MOUSE_BTN_LEFT) {
            draw_brush(x, y, WHITE);
            draw_cursor(prev_x, prev_y);
            draw_cursor(prev_x, prev_y);
        } else if (ms.buttons & MOUSE_BTN_RIGHT) {
            draw_brush(x, y, BLACK);
            draw_cursor(prev_x, prev_y);
            draw_cursor(prev_x, prev_y);
        }

        delay(8);
    }

    draw_cursor(prev_x, prev_y);
    clear();
    return 0;
}
