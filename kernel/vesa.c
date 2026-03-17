#include "vesa.h"
#include "io.h"
#include "string.h"

#define LFB_ADDR_PTR ((volatile uint32_t*)0x8000)
#define VIDEO_SIZE   (800*600)

static uint8_t* framebuffer = (uint8_t*)0xFD000000;
static int cursor_x = 0;
static int cursor_y = 0;
static uint8_t fg_color = COLOR_WHITE;
static uint8_t bg_color = COLOR_BLACK;

// =====================================
// Шрифт 8×16 (ASCII 32-126)
// =====================================
#include "font_data.h"


// инициализация через BIOS
int vesa_init(void) {
    uint32_t lfb = *LFB_ADDR_PTR;
    if (lfb != 0 && lfb != 0xFFFFFFFF) {
        framebuffer = (uint8_t*)lfb;
    }

    vesa_init_palette();
    vesa_clear(COLOR_BLACK);
    return 0;
}

// =====================================
// Палитра
// =====================================

void vesa_set_palette(uint8_t index, uint8_t r, uint8_t g, uint8_t b) {
    outb(0x3C8, index);
    outb(0x3C9, r >> 2);
    outb(0x3C9, g >> 2);
    outb(0x3C9, b >> 2);
}

void vesa_get_palette(uint8_t index, uint8_t* r, uint8_t* g, uint8_t* b) {
    outb(0x3C7, index);
    if(r) *r = inb(0x3C9) << 2;
    if(g) *g = inb(0x3C9) << 2;
    if(b) *b = inb(0x3C9) << 2;
}

void vesa_init_palette(void) {
    // Стандартная VGA палитра (16 цветов)
    vesa_set_palette(0,  0x00, 0x00, 0x00);  // Black
    vesa_set_palette(1,  0x00, 0x00, 0xAA);  // Blue
    vesa_set_palette(2,  0x00, 0xAA, 0x00);  // Green
    vesa_set_palette(3,  0x00, 0xAA, 0xAA);  // Cyan
    vesa_set_palette(4,  0xAA, 0x00, 0x00);  // Red
    vesa_set_palette(5,  0xAA, 0x00, 0xAA);  // Magenta
    vesa_set_palette(6,  0xAA, 0x55, 0x00);  // Brown
    vesa_set_palette(7,  0xAA, 0xAA, 0xAA);  // Light Grey
    vesa_set_palette(8,  0x55, 0x55, 0x55);  // Dark Grey
    vesa_set_palette(9,  0x55, 0x55, 0xFF);  // Light Blue
    vesa_set_palette(10, 0x55, 0xFF, 0x55);  // Light Green
    vesa_set_palette(11, 0x55, 0xFF, 0xFF);  // Light Cyan
    vesa_set_palette(12, 0xFF, 0x55, 0x55);  // Light Red
    vesa_set_palette(13, 0xFF, 0x55, 0xFF);  // Light Magenta
    vesa_set_palette(14, 0xFF, 0xFF, 0x55);  // Yellow
    vesa_set_palette(15, 0xFF, 0xFF, 0xFF);  // White
    
    // Градиент серого 16-31
    for(int i = 16; i < 32; i++) {
        uint8_t v = (i - 16) * 16;
        vesa_set_palette(i, v, v, v);
    }
    
    // Остальные цвета
    for(int i = 32; i < 256; i++) {
        uint8_t r = ((i - 32) % 6) * 51;
        uint8_t g = (((i - 32) / 6) % 6) * 51;
        uint8_t b = (((i - 32) / 36) % 6) * 51;
        vesa_set_palette(i, r, g, b);
    }
}

// Пиксели
void vesa_pixel(int x, int y, uint8_t color) {
    if(x < 0 || x >= SCREEN_WIDTH || y < 0 || y >= SCREEN_HEIGHT) return;
    framebuffer[y * SCREEN_WIDTH + x] = color;
}

uint8_t vesa_getpixel(int x, int y) {
    if(x < 0 || x >= SCREEN_WIDTH || y < 0 || y >= SCREEN_HEIGHT) return 0;
    return framebuffer[y * SCREEN_WIDTH + x];
}

void vesa_clear(uint8_t color) {
    memset(framebuffer, color, VIDEO_SIZE);
}

// Примитивы
void vesa_hline(int x, int y, int len, uint8_t color) {
    if(y < 0 || y >= SCREEN_HEIGHT) return;
    int x1 = x < 0 ? 0 : x;
    int x2 = x + len > SCREEN_WIDTH ? SCREEN_WIDTH : x + len;
    memset(&framebuffer[y * SCREEN_WIDTH + x1], color, x2 - x1);
}

void vesa_vline(int x, int y, int len, uint8_t color) {
    if(x < 0 || x >= SCREEN_WIDTH) return;
    int y1 = y < 0 ? 0 : y;
    int y2 = y + len > SCREEN_HEIGHT ? SCREEN_HEIGHT : y + len;
    for(int i = y1; i < y2; i++) {
        framebuffer[i * SCREEN_WIDTH + x] = color;
    }
}

void vesa_line(int x0, int y0, int x1, int y1, uint8_t color) {
    int dx = x1 - x0;
    int dy = y1 - y0;
    if(dx < 0) dx = -dx;
    if(dy < 0) dy = -dy;
    
    int sx = x0 < x1 ? 1 : -1;
    int sy = y0 < y1 ? 1 : -1;
    int err = dx - dy;
    
    while(1) {
        vesa_pixel(x0, y0, color);
        if(x0 == x1 && y0 == y1) break;
        int e2 = 2 * err;
        if(e2 > -dy) { err -= dy; x0 += sx; }
        if(e2 < dx)  { err += dx; y0 += sy; }
    }
}

void vesa_rect(int x, int y, int w, int h, uint8_t color) {
    vesa_hline(x, y, w, color);
    vesa_hline(x, y + h - 1, w, color);
    vesa_vline(x, y, h, color);
    vesa_vline(x + w - 1, y, h, color);
}

void vesa_fillrect(int x, int y, int w, int h, uint8_t color) {
    for(int i = 0; i < h; i++) {
        vesa_hline(x, y + i, w, color);
    }
}

void vesa_circle(int cx, int cy, int r, uint8_t color) {
    int x = 0, y = r;
    int d = 3 - 2 * r;
    while(y >= x) {
        vesa_pixel(cx + x, cy + y, color);
        vesa_pixel(cx - x, cy + y, color);
        vesa_pixel(cx + x, cy - y, color);
        vesa_pixel(cx - x, cy - y, color);
        vesa_pixel(cx + y, cy + x, color);
        vesa_pixel(cx - y, cy + x, color);
        vesa_pixel(cx + y, cy - x, color);
        vesa_pixel(cx - y, cy - x, color);
        x++;
        if(d > 0) { y--; d = d + 4 * (x - y) + 10; }
        else d = d + 4 * x + 6;
    }
}

void vesa_fillcircle(int cx, int cy, int r, uint8_t color) {
    for(int y = -r; y <= r; y++) {
        for(int x = -r; x <= r; x++) {
            if(x*x + y*y <= r*r) {
                vesa_pixel(cx + x, cy + y, color);
            }
        }
    }
}

// =====================================
// Текст
// =====================================

// символы 0x80-0xFF

/* noinline гарантирует что компилятор не заинлайнит и не соптимизирует */
void vesa_putchar(int x, int y, char c, uint8_t fg, uint8_t bg) {
    unsigned char uc = (unsigned char)c;
    /* Явный volatile доступ — исключаем любую оптимизацию индексации */
    volatile uint8_t *base = (volatile uint8_t*)font_data;
    volatile uint8_t *glyph = base + uc * 16;
    
    for(int row = 0; row < 16; row++) {
        uint8_t bits = glyph[row];
        for(int col = 0; col < 8; col++) {
            uint8_t color = (bits & (0x80 >> col)) ? fg : bg;
            vesa_pixel(x + col, y + row, color);
        }
    }
}

void vesa_puts(int x, int y, const char* str, uint8_t fg, uint8_t bg) {
    int px = x;
    while(*str) {
        vesa_putchar(px, y, *str, fg, bg);
        px += 8;
        str++;
    }
}

void vesa_print(const char* str) {
    while(*str) {
        vesa_print_char(*str);
        str++;
    }
}

void vesa_println(const char* str) {
    vesa_print(str);
    vesa_print_char('\n');
}

void vesa_print_char(char c) {
    if(c == '\n') {
        cursor_x = 0;
        cursor_y++;
        if(cursor_y >= TEXT_ROWS) {
            memmove(framebuffer, framebuffer + SCREEN_WIDTH * 16, 
                    SCREEN_WIDTH * (SCREEN_HEIGHT - 16));
            memset(framebuffer + SCREEN_WIDTH * (SCREEN_HEIGHT - 16), 
                   bg_color, SCREEN_WIDTH * 16);
            cursor_y = TEXT_ROWS - 1;
        }
    } else if(c == '\b') {
        if(cursor_x > 0) {
            cursor_x--;
            vesa_putchar(cursor_x * 8, cursor_y * 16, ' ', fg_color, bg_color);
        }
    } else if((unsigned char)c >= 32) {
        vesa_putchar(cursor_x * 8, cursor_y * 16, c, fg_color, bg_color);
        cursor_x++;
        if(cursor_x >= TEXT_COLS) {
            cursor_x = 0;
            cursor_y++;
            if(cursor_y >= TEXT_ROWS) {
                memmove(framebuffer, framebuffer + SCREEN_WIDTH * 16, 
                        SCREEN_WIDTH * (SCREEN_HEIGHT - 16));
                memset(framebuffer + SCREEN_WIDTH * (SCREEN_HEIGHT - 16), 
                       bg_color, SCREEN_WIDTH * 16);
                cursor_y = TEXT_ROWS - 1;
            }
        }
    }
}

void vesa_backspace(void) {
    vesa_print_char('\b');
}

void vesa_set_cursor(int x, int y) {
    cursor_x = x;
    cursor_y = y;
}

void vesa_get_cursor(int* x, int* y) {
    if(x) *x = cursor_x;
    if(y) *y = cursor_y;
}

void vesa_set_color(uint8_t fg, uint8_t bg) {
    fg_color = fg;
    bg_color = bg;
}

void vesa_vsync(void) {
    while(inb(0x3DA) & 8);
    while(!(inb(0x3DA) & 8));
}

int vesa_width(void) { return SCREEN_WIDTH; }
int vesa_height(void) { return SCREEN_HEIGHT; }
uint8_t* vesa_get_framebuffer(void) { return framebuffer; }