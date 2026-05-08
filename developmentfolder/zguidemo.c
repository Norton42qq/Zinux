#include "zxe_api.h"

static const uint32_t ICON[8*8] = {
    0,         LIGHT_GREEN, 0,         LIGHT_GREEN, 0,         LIGHT_GREEN, 0,         0,
    LIGHT_GREEN,YELLOW,     LIGHT_GREEN,YELLOW,      LIGHT_GREEN,YELLOW,     LIGHT_GREEN,0,
    0,         LIGHT_GREEN, YELLOW,     YELLOW,      YELLOW,     LIGHT_GREEN,0,         0,
    LIGHT_GREEN,YELLOW,     YELLOW,     WHITE,       YELLOW,     YELLOW,     LIGHT_GREEN,0,
    0,         LIGHT_GREEN, YELLOW,     YELLOW,      YELLOW,     LIGHT_GREEN,0,         0,
    LIGHT_GREEN,YELLOW,     LIGHT_GREEN,YELLOW,      LIGHT_GREEN,YELLOW,     LIGHT_GREEN,0,
    0,         LIGHT_GREEN, 0,         LIGHT_GREEN, 0,         LIGHT_GREEN, 0,         0,
    0,         0,           0,         0,           0,         0,           0,         0,
};

static void draw_page_graphics(void) {
    clear();
    int sw = screen_w();
    int sh = screen_h();

    // Заголовок
    fillrect(0, 0, sw, 20, RGB(20, 20, 60));
    gfx_text(8, 2, "ZINUX GUI DEMO - Pixel Graphics", WHITE, RGB(20,20,60));

    // Цветные прямоугольники
    int colors[] = {RED, GREEN, BLUE, YELLOW, MAGENTA, CYAN,
                    LIGHT_RED, LIGHT_GREEN, LIGHT_BLUE, LIGHT_CYAN,
                    LIGHT_MAGENTA, YELLOW, WHITE, LIGHT_GREY, DARK_GREY};
    for (int i = 0; i < 15; i++) {
        fillrect(30 + i*50, 40, 44, 30, colors[i]);
        rect(30 + i*50, 40, 44, 30, WHITE);
    }

    // Линии веером
    int cx = 100, cy = 160;
    for (int a = 0; a < 16; a++) {
        uint32_t c = RGB(a*16, 255 - a*12, 128);
        line(cx, cy, cx + a*18, cy + 80, c);
    }

    // Круги
    for (int r = 5; r <= 50; r += 9) {
        uint32_t c = RGB(r*4, 200 - r*2, 255 - r*3);
        circle(500, 200, r, c);
    }
    fillcircle(650, 200, 35, LIGHT_CYAN);
    fillcircle(650, 200, 20, BLUE);
    fillcircle(650, 200, 8,  WHITE);

    // Градиент
    for (int x = 0; x < 200; x++) {
        uint32_t c = RGB(x*255/200, 100, 255 - x*255/200);
        line(800 + x, 40, 800 + x, 110, c);
    }
    rect(800, 40, 200, 70, WHITE);
    gfx_text(800, 120, "gradient", LIGHT_GREY, BLACK);

    // Blit спрайт (увеличенный 4x)
    for (int row = 0; row < 8; row++)
        for (int col = 0; col < 8; col++)
            if (ICON[row*8+col])
                fillrect(900 + col*8, 150 + row*8, 8, 8, ICON[row*8+col]);
    rect(900, 150, 64, 64, DARK_GREY);
    gfx_text(896, 220, "blit 4x", LIGHT_GREY, BLACK);

    // Инструкция
    fillrect(0, sh-24, sw, 24, RGB(20,20,20));
    gfx_text(8, sh-20, "[ 1 ] Graphics  [ 2 ] Widgets  [ 3 ] Colors  [ ESC ] Quit", DARK_GREY, RGB(20,20,20));
}

static void draw_page_widgets(void) {
    clear();
    int sw = screen_w();
    int sh = screen_h();

    fillrect(0, 0, sw, 20, RGB(40,20,20));
    gfx_text(8, 2, "ZINUX GUI DEMO - Widgets", WHITE, RGB(40,20,20));

    // Окно
    window(5, 2, 35, 16, " Settings ");
    label(7, 4,  "System options:", LIGHT_CYAN);
    checkbox(7, 6,  "Enable sounds",   1);
    checkbox(7, 8,  "Show clock",      1);
    checkbox(7, 10, "Dark mode",       0);
    checkbox(7, 12, "Auto-save",       1);
    button(7, 15, " Apply ", 1);
    button(18, 15, " Cancel ", 0);

    // Прогресс-бары
    window(42, 2, 35, 16, " Progress ");
    label(44, 4, "CPU:", LIGHT_GREEN);   
    gotoxy(50, 4); color(LIGHT_GREEN, BLACK);
    print("72%");
    gotoxy(44, 5); progress(44, 5, 30, 72);
    label(44, 7, "RAM:", YELLOW);
    gotoxy(50, 7); color(YELLOW, BLACK);
    print("45%");
    gotoxy(44, 8); progress(44, 8, 30, 45);
    label(44, 10, "Disk:", LIGHT_RED);
    gotoxy(50, 10); color(LIGHT_RED, BLACK);
    print("91%");
    gotoxy(44, 11); progress(44, 11, 30, 91);

    // Табы
    const char* tabs[] = { "Files", "Network", "Display", "About" };
    tabbar(5, 19, 60, tabs, 4, 2);
    fill(5, 21, 60, 8, ' ', WHITE, RGB(10,10,40));
    box(5, 21, 60, 8, DARK_GREY, RGB(10,10,40));
    label(7, 22, "Display tab content:", LIGHT_CYAN);
    checkbox(7, 24, "1920x1080 @ 32bpp", 1);
    checkbox(7, 26, "VSync",             0);

    // Меню
    const char* menu_items[] = { "File", "Edit", "View", "Help" };
    menubar(sh/16 - 3, menu_items, 4, 0);

    fillrect(0, sh-24, sw, 24, RGB(20,20,20));
    gfx_text(8, sh-20, "[ 1 ] Graphics  [ 2 ] Widgets  [ 3 ] Colors  [ ESC ] Quit", DARK_GREY, RGB(20,20,20));
}

static void draw_page_colors(void) {
    clear();
    int sw = screen_w();
    int sh = screen_h();

    fillrect(0, 0, sw, 20, RGB(20, 40, 20));
    gfx_text(8, 2, "ZINUX GUI DEMO - Color Palette", WHITE, RGB(20,40,20));

    // Стандартные 16 цветов
    uint32_t pal16[] = {
        BLACK, BLUE, GREEN, CYAN, RED, MAGENTA, BROWN, LIGHT_GREY,
        DARK_GREY, LIGHT_BLUE, LIGHT_GREEN, LIGHT_CYAN,
        LIGHT_RED, LIGHT_MAGENTA, YELLOW, WHITE
    };
    const char* names16[] = {
        "BLACK","BLUE","GREEN","CYAN","RED","MAGENTA","BROWN","L.GREY",
        "D.GREY","L.BLUE","L.GREEN","L.CYAN","L.RED","L.MAGENTA","YELLOW","WHITE"
    };
    for (int i = 0; i < 16; i++) {
        int col = i % 8;
        int row = i / 8;
        fillrect(20 + col*100, 35 + row*50, 80, 30, pal16[i]);
        rect(20 + col*100, 35 + row*50, 80, 30, DARK_GREY);
        gfx_text(20 + col*100 + 4, 35 + row*50 + 36, names16[i], LIGHT_GREY, BLACK);
    }

    // RGB куб срез
    gfx_text(20, 150, "RGB slices:", LIGHT_CYAN, BLACK);
    for (int g = 0; g < 6; g++) {
        for (int r = 0; r < 32; r++) {
            for (int b = 0; b < 32; b++) {
                uint32_t c = RGB(r*8, g*51, b*8);
                pixel(20 + g*70 + r, 165 + b, c);
            }
        }
        char buf[4];
        buf[0] = 'G';
        buf[1] = '=';
        buf[2] = '0' + g;
        buf[3] = 0;
        gfx_text(20 + g*70, 200, buf, LIGHT_GREY, BLACK);
    }

    // Градиенты
    gfx_text(20, 220, "Gradients:", LIGHT_CYAN, BLACK);
    for (int x = 0; x < 400; x++) {
        line(20+x, 235, 20+x, 260, RGB(x*255/400, 0, 0));
        line(20+x, 265, 20+x, 290, RGB(0, x*255/400, 0));
        line(20+x, 295, 20+x, 320, RGB(0, 0, x*255/400));
        line(20+x, 325, 20+x, 350, RGB(x*255/400, x*255/400, 0));
    }

    fillrect(0, sh-24, sw, 24, RGB(20,20,20));
    gfx_text(8, sh-20, "[ 1 ] Graphics  [ 2 ] Widgets  [ 3 ] Colors  [ ESC ] Quit", DARK_GREY, RGB(20,20,20));
}

int main(int argc, char* argv[]) {
    (void)argc; (void)argv;

    int page = 1;
    draw_page_graphics();

    while (1) {
        char k = waitkey();
        if (k == KEY_ESC) break;
        if (k == '1') { page = 1; draw_page_graphics(); }
        if (k == '2') { page = 2; draw_page_widgets();  }
        if (k == '3') { page = 3; draw_page_colors();   }
    }

    clear();
    return 0;
}