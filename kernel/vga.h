#ifndef VGA_H
#define VGA_H

#include "vesa.h"

typedef enum {
    VGA_COLOR_BLACK         = COLOR_BLACK,
    VGA_COLOR_BLUE          = COLOR_BLUE,
    VGA_COLOR_GREEN         = COLOR_GREEN,
    VGA_COLOR_CYAN          = COLOR_CYAN,
    VGA_COLOR_RED           = COLOR_RED,
    VGA_COLOR_MAGENTA       = COLOR_MAGENTA,
    VGA_COLOR_BROWN         = COLOR_BROWN,
    VGA_COLOR_LIGHT_GREY    = COLOR_LIGHT_GREY,
    VGA_COLOR_DARK_GREY     = COLOR_DARK_GREY,
    VGA_COLOR_LIGHT_BLUE    = COLOR_LIGHT_BLUE,
    VGA_COLOR_LIGHT_GREEN   = COLOR_LIGHT_GREEN,
    VGA_COLOR_LIGHT_CYAN    = COLOR_LIGHT_CYAN,
    VGA_COLOR_LIGHT_RED     = COLOR_LIGHT_RED,
    VGA_COLOR_LIGHT_MAGENTA = COLOR_LIGHT_MAGENTA,
    VGA_COLOR_YELLOW        = COLOR_YELLOW,
    VGA_COLOR_WHITE         = COLOR_WHITE
} vga_color_t;

// Размеры экрана в символах
#define SCREEN_COLS TEXT_COLS
#define SCREEN_ROWS TEXT_ROWS

static inline void vga_init(void)               { vesa_init(); }
static inline void vga_clear(void)              { vesa_clear(COLOR_BLACK); }
static inline void vga_putchar(char c)          { vesa_print_char(c); }
static inline void vga_puts(const char* s)      { vesa_print(s); }
static inline void vga_backspace(void)          { vesa_backspace(); }
static inline void vga_set_cursor(int x, int y) { vesa_set_cursor(x, y); }
static inline void vga_get_cursor(int* x, int* y) { vesa_get_cursor(x, y); }

static inline void vga_set_color(vga_color_t fg, vga_color_t bg) {
    vesa_set_color((uint8_t)fg, (uint8_t)bg);
}

static inline void vga_put_dec(uint32_t n) {
    if (n == 0) { vesa_print_char('0'); return; }
    char buf[12]; int i = 0;
    while (n > 0) { buf[i++] = '0' + (n % 10); n /= 10; }
    while (i > 0) vesa_print_char(buf[--i]);
}

static inline void vga_put_hex(uint32_t n) {
    const char hex[] = "0123456789ABCDEF";
    vesa_print("0x");
    for (int i = 28; i >= 0; i -= 4)
        vesa_print_char(hex[(n >> i) & 0xF]);
}

#endif