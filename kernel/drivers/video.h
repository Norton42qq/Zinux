#ifndef VIDEO_H
#define VIDEO_H
#include <stdint.h>

#define VIDEO_INFO_ADDR  0x8000

#define VIDEO_BPP_8   8
#define VIDEO_BPP_15  15
#define VIDEO_BPP_16  16
#define VIDEO_BPP_24  24
#define VIDEO_BPP_32  32

typedef struct {
    uint32_t lfb_addr;      // физический адрес линейного фреймбуфера
    uint16_t width;         // ширина в пикселях
    uint16_t height;        // высота в пикселях
    uint32_t pitch;         // байт на строку (может быть > width*bpp/8)
    uint8_t  bpp;           // бит на пиксель: 8,15,16,24,32
    uint8_t  red_pos;       // позиция красного канала (биты)
    uint8_t  red_size;
    uint8_t  green_pos;
    uint8_t  green_size;
    uint8_t  blue_pos;
    uint8_t  blue_size;
    uint8_t  reserved;      // выравнивание
} __attribute__((packed)) VideoInfo;

// Цветовые константы
#define RGB(r,g,b)          (((uint32_t)(r)<<16)|((uint32_t)(g)<<8)|(b))

#define COLOR_BLACK         RGB(  0,  0,  0)
#define COLOR_BLUE          RGB(  0,  0,170)
#define COLOR_GREEN         RGB(  0,170,  0)
#define COLOR_CYAN          RGB(  0,170,170)
#define COLOR_RED           RGB(170,  0,  0)
#define COLOR_MAGENTA       RGB(170,  0,170)
#define COLOR_BROWN         RGB(170, 85,  0)
#define COLOR_LIGHT_GREY    RGB(170,170,170)
#define COLOR_DARK_GREY     RGB( 85, 85, 85)
#define COLOR_LIGHT_BLUE    RGB( 85, 85,255)
#define COLOR_LIGHT_GREEN   RGB( 85,255, 85)
#define COLOR_LIGHT_CYAN    RGB( 85,255,255)
#define COLOR_LIGHT_RED     RGB(255, 85, 85)
#define COLOR_LIGHT_MAGENTA RGB(255, 85,255)
#define COLOR_YELLOW        RGB(255,255, 85)
#define COLOR_WHITE         RGB(255,255,255)

// Текстовый слой
extern int VIDEO_TEXT_COLS;
extern int VIDEO_TEXT_ROWS;


int  video_init(void);
const VideoInfo* video_get_info(void);

void     video_pixel      (int x, int y, uint32_t color);
uint32_t video_getpixel   (int x, int y);
void     video_clear      (uint32_t color);
void     video_hline      (int x, int y, int len, uint32_t color);
void     video_vline      (int x, int y, int len, uint32_t color);
void     video_line       (int x0, int y0, int x1, int y1, uint32_t color);
void     video_rect       (int x, int y, int w, int h, uint32_t color);
void     video_fillrect   (int x, int y, int w, int h, uint32_t color);
void     video_circle     (int cx, int cy, int r, uint32_t color);
void     video_fillcircle (int cx, int cy, int r, uint32_t color);

void     video_putchar    (int col, int row, char c, uint32_t fg, uint32_t bg);
void     video_puts       (int col, int row, const char* str, uint32_t fg, uint32_t bg);
void     video_print      (const char* str);
void     video_println    (const char* str);
void     video_print_char (char c);
void     video_backspace  (void);
void     video_set_cursor (int col, int row);
void     video_get_cursor (int* col, int* row);
void     video_set_color  (uint32_t fg, uint32_t bg);
void     video_vsync      (void);
int      video_width      (void);
int      video_height     (void);
void     video_put_dec    (uint32_t n);
void     video_put_hex    (uint32_t n);

// Обратная совместимость
#define vesa_init()                video_init()
#define vesa_clear(c)              video_clear(c)
#define vesa_pixel(x,y,c)          video_pixel(x,y,c)
#define vesa_hline(x,y,l,c)        video_hline(x,y,l,c)
#define vesa_vline(x,y,l,c)        video_vline(x,y,l,c)
#define vesa_line(x0,y0,x1,y1,c)   video_line(x0,y0,x1,y1,c)
#define vesa_rect(x,y,w,h,c)       video_rect(x,y,w,h,c)
#define vesa_fillrect(x,y,w,h,c)   video_fillrect(x,y,w,h,c)
#define vesa_circle(cx,cy,r,c)     video_circle(cx,cy,r,c)
#define vesa_fillcircle(cx,cy,r,c) video_fillcircle(cx,cy,r,c)
#define vesa_putchar(x,y,ch,f,b)   video_putchar(x,y,ch,f,b)
#define vesa_puts(x,y,s,f,b)       video_puts(x,y,s,f,b)
#define vesa_print(s)              video_print(s)
#define vesa_println(s)            video_println(s)
#define vesa_print_char(c)         video_print_char(c)
#define vesa_backspace()           video_backspace()
#define vesa_set_cursor(x,y)       video_set_cursor(x,y)
#define vesa_get_cursor(x,y)       video_get_cursor(x,y)
#define vesa_set_color(f,b)        video_set_color(f,b)
#define vesa_vsync()               video_vsync()
#define vesa_width()               video_width()
#define vesa_height()              video_height()
#define vesa_put_dec(n)            video_put_dec(n)
#define vesa_put_hex(n)            video_put_hex(n)

#endif