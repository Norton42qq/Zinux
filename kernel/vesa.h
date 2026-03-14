#ifndef VESA_H
#define VESA_H

#include <stdint.h>

// Параметры экрана
#define SCREEN_WIDTH  800
#define SCREEN_HEIGHT 600
#define SCREEN_BPP    8      // 8 бит = 256 цветов

// Текстовый режим эмуляции
#define TEXT_COLS     100    // 800/8
#define TEXT_ROWS     37     // 600/16

// Стандартная VGA палитра
#define COLOR_BLACK         0
#define COLOR_BLUE          1
#define COLOR_GREEN         2
#define COLOR_CYAN          3
#define COLOR_RED           4
#define COLOR_MAGENTA       5
#define COLOR_BROWN         6
#define COLOR_LIGHT_GREY    7
#define COLOR_DARK_GREY     8
#define COLOR_LIGHT_BLUE    9
#define COLOR_LIGHT_GREEN   10
#define COLOR_LIGHT_CYAN    11
#define COLOR_LIGHT_RED     12
#define COLOR_LIGHT_MAGENTA 13
#define COLOR_YELLOW        14
#define COLOR_WHITE         15

// Инициализация
int  vesa_init(void);                   // 0=OK, -1=error
void vesa_clear(uint8_t color);

// Пиксели
void    vesa_pixel(int x, int y, uint8_t color);
uint8_t vesa_getpixel(int x, int y);

// Примитивы
void vesa_hline(int x, int y, int len, uint8_t color);
void vesa_vline(int x, int y, int len, uint8_t color);
void vesa_line(int x0, int y0, int x1, int y1, uint8_t color);
void vesa_rect(int x, int y, int w, int h, uint8_t color);
void vesa_fillrect(int x, int y, int w, int h, uint8_t color);
void vesa_circle(int cx, int cy, int r, uint8_t color);
void vesa_fillcircle(int cx, int cy, int r, uint8_t color);

// Текст
void vesa_putchar(int x, int y, char c, uint8_t fg, uint8_t bg);
void vesa_puts(int x, int y, const char* str, uint8_t fg, uint8_t bg);
void vesa_print(const char* str);       // Вывод с автопереносом
void vesa_println(const char* str);
void vesa_print_char(char c);
void vesa_backspace(void);
void vesa_set_cursor(int x, int y);
void vesa_get_cursor(int* x, int* y);
void vesa_set_color(uint8_t fg, uint8_t bg);

// Палитра
void vesa_set_palette(uint8_t index, uint8_t r, uint8_t g, uint8_t b);
void vesa_get_palette(uint8_t index, uint8_t* r, uint8_t* g, uint8_t* b);
void vesa_init_palette(void);          // Стандартная VGA палитра

// Утилиты
void vesa_vsync(void);
int  vesa_width(void);
int  vesa_height(void);
uint8_t* vesa_get_framebuffer(void);

#endif