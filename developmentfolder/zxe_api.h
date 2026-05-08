#ifndef ZXE_API_H
#define ZXE_API_H

typedef unsigned char  uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int   uint32_t;
#define NULL ((void*)0)

#define RGB(r,g,b) (((uint32_t)(r)<<16)|((uint32_t)(g)<<8)|(b))

// -- Клавиши --
#define KEY_UP        ((char)128)
#define KEY_DOWN      ((char)129)
#define KEY_LEFT      ((char)130)
#define KEY_RIGHT     ((char)131)
#define KEY_ESC       27
#define KEY_ENTER     '\n'
#define KEY_SPACE     ' '
#define KEY_BACKSPACE '\b'
#define KEY_TAB       '\t'
#define KEY_F1        ((char)132)
#define KEY_F2        ((char)133)
#define KEY_F3        ((char)134)
#define KEY_F4        ((char)135)

// -- Цвета 0x00RRGGBB --
#define BLACK         0x00000000
#define BLUE          0x000000AA
#define GREEN         0x0000AA00
#define CYAN          0x0000AAAA
#define RED           0x00AA0000
#define MAGENTA       0x00AA00AA
#define BROWN         0x00AA5500
#define LIGHT_GREY    0x00AAAAAA
#define DARK_GREY     0x00555555
#define LIGHT_BLUE    0x005555FF
#define LIGHT_GREEN   0x0055FF55
#define LIGHT_CYAN    0x0055FFFF
#define LIGHT_RED     0x00FF5555
#define LIGHT_MAGENTA 0x00FF55FF
#define YELLOW        0x00FFFF55
#define WHITE         0x00FFFFFF

// -- Псевдографика --
#define CHAR_HLINE    '\xC4'  //
#define CHAR_VLINE    '\xB3'  // │
#define CHAR_TL       '\xDA'  // ┌
#define CHAR_TR       '\xBF'  // ┐
#define CHAR_BL       '\xC0'  // └
#define CHAR_BR       '\xD9'  // ┘
#define CHAR_SHADE    '\xB0'  // ░
#define CHAR_BLOCK    '\xDB'  // █

// -- Экран --
#define SCREEN_COLS  80
#define SCREEN_ROWS  30

// -- Мышь (кнопки) --
#define MOUSE_BTN_LEFT   (1 << 0)
#define MOUSE_BTN_RIGHT  (1 << 1)
#define MOUSE_BTN_MIDDLE (1 << 2)

// -- API Таблица --
#define API_TABLE_ADDRESS 0x100000

// Системная информация
typedef struct {
    char     username[32];
    char     hostname[32];
    char     os_name[32];
    char     os_version[16];
    char     cwd[64];
    uint32_t mem_kb;
} __attribute__((packed)) sys_info_t;

// Состояние мыши
typedef struct {
    int     x;
    int     y;
    uint8_t buttons;
    int     wheel;
} __attribute__((packed)) mouse_state_t;

typedef struct {
    // Основные команды
    void     (*print)      (const char* str);
    void     (*println)    (const char* str);
    void     (*print_char) (char c);
    void     (*print_int)  (int value);
    void     (*print_hex)  (uint32_t value);
    void     (*set_color)  (uint32_t fg, uint32_t bg);
    char     (*read_char)  (void);
    void     (*read_line)  (char* buf, int max_len);
    void     (*clear_screen)(void);
    void     (*set_cursor)  (int x, int y);
    void     (*get_cursor)  (int* x, int* y);
    void     (*draw_box)    (int x, int y, int w, int h, uint32_t fg, uint32_t bg);
    void     (*draw_box_titled)(int x, int y, int w, int h, const char* title, uint32_t fg, uint32_t bg);
    void     (*fill_rect)   (int x, int y, int w, int h, char ch, uint32_t fg, uint32_t bg);
    void     (*draw_hline)  (int x, int y, int len, uint32_t fg, uint32_t bg);
    void     (*draw_vline)  (int x, int y, int len, uint32_t fg, uint32_t bg);
    void     (*draw_window) (int x, int y, int w, int h, const char* title);
    void     (*draw_button) (int x, int y, const char* text, int selected);
    void     (*draw_input)  (int x, int y, int w, const char* text, int active);
    void     (*draw_progress)(int x, int y, int w, int percent);
    int      (*str_compare) (const char* s1, const char* s2);
    int      (*str_length)  (const char* str);
    void     (*str_copy)    (char* dest, const char* src);
    void     (*mem_copy)    (void* dest, const void* src, uint32_t size);
    void     (*mem_set)     (void* dest, uint8_t val, uint32_t size);
    void     (*get_time)    (uint8_t* h, uint8_t* m, uint8_t* s);
    void     (*get_date)    (uint8_t* d, uint8_t* m, uint16_t* y);
    void     (*delay)       (uint32_t ms);
    int      (*file_exists) (const char* fn);
    int      (*file_read)   (const char* fn, void* buf, uint32_t max);
    uint32_t (*file_size)   (const char* fn);
    int      (*file_write)  (const char* fn, const void* buf, uint32_t size);
    int      (*has_key)     (void);
    char     (*get_key)     (void);
    void     (*clear_keys)  (void);
    void     (*get_sysinfo) (sys_info_t* info);
    void     (*set_username)(const char* name);
    void     (*set_hostname)(const char* name);
    // Пиксельная графика
    void     (*gfx_pixel)     (int x, int y, uint32_t color);
    void     (*gfx_line)      (int x0, int y0, int x1, int y1, uint32_t color);
    void     (*gfx_rect)      (int x, int y, int w, int h, uint32_t color);
    void     (*gfx_fillrect)  (int x, int y, int w, int h, uint32_t color);
    void     (*gfx_circle)    (int cx, int cy, int r, uint32_t color);
    void     (*gfx_fillcircle)(int cx, int cy, int r, uint32_t color);
    void     (*gfx_char)      (int x, int y, char c, uint32_t fg, uint32_t bg);
    void     (*gfx_text)      (int x, int y, const char* s, uint32_t fg, uint32_t bg);
    int      (*gfx_width)     (void);
    int      (*gfx_height)    (void);
    void     (*gfx_blit)      (int x, int y, int w, int h, const uint32_t* pixels);
    // GUI виджеты
    void     (*gui_label)     (int x, int y, const char* text, uint32_t color);
    void     (*gui_checkbox)  (int x, int y, const char* label, int checked);
    void     (*gui_menubar)   (int y, const char** items, int count, int selected);
    void     (*gui_tabbar)    (int x, int y, int w, const char** tabs, int count, int active);
    void     (*gui_draw_cursor)(int x, int y);
    void     (*gui_listbox)    (int x, int y, int w, int h, const char** items, int count, int selected);
    void     (*gui_scrollbar)  (int x, int y, int h, int total, int visible, int pos);
    void     (*gui_popup_menu) (int x, int y, const char** items, int count, int selected);
    void     (*gui_titlebar)   (int x, int y, int w, const char* title, int active);
    // Строковые утилиты
    void     (*str_concat)    (char* dst, const char* src);
    int      (*str_ncompare)  (const char* a, const char* b, int n);
    int      (*str_to_int)    (const char* s);
    void     (*int_to_str)    (int v, char* buf);
    // Мышь
    void     (*mouse_get_state)(mouse_state_t* out);
    void     (*mouse_set_pos)  (int x, int y);
    void     (*mouse_set_sens) (int sx, int sy);    

} __attribute__((packed)) api_table_t;

#include "../syscall_api_ids.h"

static inline uint32_t __syscall_api(uint32_t fn_id, const uint32_t* args) {
    uint32_t ret;
    __asm__ volatile (
        "int $0x80"
        : "=a"(ret)
        : "a"(3), "b"(fn_id), "c"(args)
        : "edx", "esi", "edi", "memory"
    );
    return ret;
}

static inline uint32_t __api0(uint32_t id) {
    const uint32_t a[1] = {0};
    return __syscall_api(id, a);
}
static inline uint32_t __api1(uint32_t id, uint32_t p1) {
    const uint32_t a[1] = {p1};
    return __syscall_api(id, a);
}
static inline uint32_t __api2(uint32_t id, uint32_t p1, uint32_t p2) {
    const uint32_t a[2] = {p1, p2};
    return __syscall_api(id, a);
}
static inline uint32_t __api3(uint32_t id, uint32_t p1, uint32_t p2, uint32_t p3) {
    const uint32_t a[3] = {p1, p2, p3};
    return __syscall_api(id, a);
}
static inline uint32_t __api4(uint32_t id, uint32_t p1, uint32_t p2, uint32_t p3, uint32_t p4) {
    const uint32_t a[4] = {p1, p2, p3, p4};
    return __syscall_api(id, a);
}
static inline uint32_t __api5(uint32_t id, uint32_t p1, uint32_t p2, uint32_t p3, uint32_t p4, uint32_t p5) {
    const uint32_t a[5] = {p1, p2, p3, p4, p5};
    return __syscall_api(id, a);
}
static inline uint32_t __api6(uint32_t id, uint32_t p1, uint32_t p2, uint32_t p3, uint32_t p4, uint32_t p5, uint32_t p6) {
    const uint32_t a[6] = {p1, p2, p3, p4, p5, p6};
    return __syscall_api(id, a);
}
static inline uint32_t __api7(uint32_t id, uint32_t p1, uint32_t p2, uint32_t p3, uint32_t p4, uint32_t p5, uint32_t p6, uint32_t p7) {
    const uint32_t a[7] = {p1, p2, p3, p4, p5, p6, p7};
    return __syscall_api(id, a);
}

// Обёртки: текст
static inline void print(const char* s)      { __api1(SYSCALL_API_PRINT, (uint32_t)s); }
static inline void println(const char* s)    { __api1(SYSCALL_API_PRINTLN, (uint32_t)s); }
static inline void putchar(char c)           { __api1(SYSCALL_API_PRINT_CHAR, (uint32_t)(uint8_t)c); }
static inline void print_int(int v)          { __api1(SYSCALL_API_PRINT_INT, (uint32_t)v); }
static inline void print_hex(uint32_t v)     { __api1(SYSCALL_API_PRINT_HEX, v); }
static inline void color(uint32_t f, uint32_t b) { __api2(SYSCALL_API_SET_COLOR, f, b); }

// Обёртки: ввод
static inline char waitkey(void)             { return (char)__api0(SYSCALL_API_READ_CHAR); }
static inline void input(char* b, int m)     { __api2(SYSCALL_API_READ_LINE, (uint32_t)b, (uint32_t)m); }
static inline int  kbhit(void)               { return (int)__api0(SYSCALL_API_HAS_KEY); }
static inline char getch(void)               { return (char)__api0(SYSCALL_API_GET_KEY); }
static inline void flushkeys(void)           { __api0(SYSCALL_API_CLEAR_KEYS); }

// Обёртки: экран
static inline void clear(void)               { __api0(SYSCALL_API_CLEAR_SCREEN); }
static inline void gotoxy(int x, int y)      { __api2(SYSCALL_API_SET_CURSOR, (uint32_t)x, (uint32_t)y); }
static inline void getxy(int* x, int* y)     { __api2(SYSCALL_API_GET_CURSOR, (uint32_t)x, (uint32_t)y); }

// Обёртки: TUI рисование
static inline void box(int x, int y, int w, int h, uint32_t fg, uint32_t bg) {
    __api6(SYSCALL_API_DRAW_BOX, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)h, fg, bg);
}
static inline void box_titled(int x, int y, int w, int h, const char* t, uint32_t fg, uint32_t bg) {
    __api7(SYSCALL_API_DRAW_BOX_TITLED, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)h, (uint32_t)t, fg, bg);
}
static inline void fill(int x, int y, int w, int h, char c, uint32_t fg, uint32_t bg) {
    __api7(SYSCALL_API_FILL_RECT, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)h, (uint32_t)(uint8_t)c, fg, bg);
}
static inline void hline(int x, int y, int len, uint32_t fg, uint32_t bg) {
    __api5(SYSCALL_API_DRAW_HLINE, (uint32_t)x, (uint32_t)y, (uint32_t)len, fg, bg);
}
static inline void vline(int x, int y, int len, uint32_t fg, uint32_t bg) {
    __api5(SYSCALL_API_DRAW_VLINE, (uint32_t)x, (uint32_t)y, (uint32_t)len, fg, bg);
}

// Обёртки: TUI компоненты
static inline void window(int x, int y, int w, int h, const char* t) {
    __api5(SYSCALL_API_DRAW_WINDOW, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)h, (uint32_t)t);
}
static inline void button(int x, int y, const char* t, int sel) {
    __api4(SYSCALL_API_DRAW_BUTTON, (uint32_t)x, (uint32_t)y, (uint32_t)t, (uint32_t)sel);
}
static inline void textbox(int x, int y, int w, const char* t, int act) {
    __api5(SYSCALL_API_DRAW_INPUT, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)t, (uint32_t)act);
}
static inline void progress(int x, int y, int w, int pct) {
    __api4(SYSCALL_API_DRAW_PROGRESS, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)pct);
}

// Обёртки: строки
static inline int  streq(const char* a, const char* b) { return (int)__api2(SYSCALL_API_STR_COMPARE, (uint32_t)a, (uint32_t)b) == 0; }
static inline int  slen(const char* s)       { return (int)__api1(SYSCALL_API_STR_LENGTH, (uint32_t)s); }
static inline void scopy(char* d, const char* s) { __api2(SYSCALL_API_STR_COPY, (uint32_t)d, (uint32_t)s); }
static inline void str_cat(char* d, const char* s)    { __api2(SYSCALL_API_STR_CONCAT, (uint32_t)d, (uint32_t)s); }
static inline int  str_ncmp(const char* a, const char* b, int n) { return (int)__api3(SYSCALL_API_STR_NCOMPARE, (uint32_t)a, (uint32_t)b, (uint32_t)n); }
static inline int  to_int(const char* s)               { return (int)__api1(SYSCALL_API_STR_TO_INT, (uint32_t)s); }
static inline void to_str(int v, char* buf)            { __api2(SYSCALL_API_INT_TO_STR, (uint32_t)v, (uint32_t)buf); }

// Обёртки: файлы
static inline int      fexists(const char* f) { return (int)__api1(SYSCALL_API_FILE_EXISTS, (uint32_t)f); }
static inline int      fread(const char* f, void* b, uint32_t m) { return (int)__api3(SYSCALL_API_FILE_READ, (uint32_t)f, (uint32_t)b, m); }
static inline uint32_t fsize(const char* f)   { return __api1(SYSCALL_API_FILE_SIZE, (uint32_t)f); }
static inline int      fwrite(const char* f, const void* b, uint32_t n) { return (int)__api3(SYSCALL_API_FILE_WRITE, (uint32_t)f, (uint32_t)b, n); }

// Обёртки: время
static inline void delay(uint32_t ms) { __api1(SYSCALL_API_DELAY, ms); }

// Обёртки: мышь
static inline void mouse_get(mouse_state_t* ms)     { __api1(SYSCALL_API_MOUSE_GET_STATE, (uint32_t)ms); }
static inline void mouse_move(int x, int y)          { __api2(SYSCALL_API_MOUSE_SET_POS, (uint32_t)x, (uint32_t)y); }
static inline void mouse_sens(int sx, int sy)        { __api2(SYSCALL_API_MOUSE_SET_SENS, (uint32_t)sx, (uint32_t)sy); }

// Проверить, нажата ли кнопка мыши
static inline int mouse_left(void) {
    mouse_state_t ms; mouse_get(&ms);
    return (ms.buttons & MOUSE_BTN_LEFT) != 0;
}
static inline int mouse_right(void) {
    mouse_state_t ms; mouse_get(&ms);
    return (ms.buttons & MOUSE_BTN_RIGHT) != 0;
}

// Обёртки: пиксельная графика
static inline void pixel(int x, int y, uint32_t c)
    { __api3(SYSCALL_API_GFX_PIXEL, (uint32_t)x, (uint32_t)y, c); }
static inline void line(int x0, int y0, int x1, int y1, uint32_t c)
    { __api5(SYSCALL_API_GFX_LINE, (uint32_t)x0, (uint32_t)y0, (uint32_t)x1, (uint32_t)y1, c); }
static inline void rect(int x, int y, int w, int h, uint32_t c)
    { __api5(SYSCALL_API_GFX_RECT, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)h, c); }
static inline void fillrect(int x, int y, int w, int h, uint32_t c)
    { __api5(SYSCALL_API_GFX_FILLRECT, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)h, c); }
static inline void circle(int cx, int cy, int r, uint32_t c)
    { __api4(SYSCALL_API_GFX_CIRCLE, (uint32_t)cx, (uint32_t)cy, (uint32_t)r, c); }
static inline void fillcircle(int cx, int cy, int r, uint32_t c)
    { __api4(SYSCALL_API_GFX_FILLCIRCLE, (uint32_t)cx, (uint32_t)cy, (uint32_t)r, c); }
static inline void gfx_char(int x, int y, char c, uint32_t fg, uint32_t bg)
    { __api5(SYSCALL_API_GFX_CHAR, (uint32_t)x, (uint32_t)y, (uint32_t)(uint8_t)c, fg, bg); }
static inline void gfx_text(int x, int y, const char* s, uint32_t fg, uint32_t bg)
    { __api5(SYSCALL_API_GFX_TEXT, (uint32_t)x, (uint32_t)y, (uint32_t)s, fg, bg); }
static inline int  screen_w(void) { return (int)__api0(SYSCALL_API_GFX_WIDTH); }
static inline int  screen_h(void) { return (int)__api0(SYSCALL_API_GFX_HEIGHT); }
static inline void blit(int x, int y, int w, int h, const uint32_t* px)
    { __api5(SYSCALL_API_GFX_BLIT, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)h, (uint32_t)px); }

// Обёртки: TUI GUI виджеты
static inline void label(int x, int y, const char* t, uint32_t c)
    { __api4(SYSCALL_API_GUI_LABEL, (uint32_t)x, (uint32_t)y, (uint32_t)t, c); }
static inline void checkbox(int x, int y, const char* t, int checked)
    { __api4(SYSCALL_API_GUI_CHECKBOX, (uint32_t)x, (uint32_t)y, (uint32_t)t, (uint32_t)checked); }
static inline void menubar(int y, const char** items, int count, int sel)
    { __api4(SYSCALL_API_GUI_MENUBAR, (uint32_t)y, (uint32_t)items, (uint32_t)count, (uint32_t)sel); }
static inline void tabbar(int x, int y, int w, const char** tabs, int count, int active)
    { __api6(SYSCALL_API_GUI_TABBAR, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)tabs, (uint32_t)count, (uint32_t)active); }
static inline void draw_cursor(int x, int y)
    { __api2(SYSCALL_API_GUI_DRAW_CURSOR, (uint32_t)x, (uint32_t)y); }
static inline void listbox(int x, int y, int w, int h, const char** items, int count, int sel)
    { __api7(SYSCALL_API_GUI_LISTBOX, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)h, (uint32_t)items, (uint32_t)count, (uint32_t)sel); }
static inline void scrollbar(int x, int y, int h, int total, int visible, int pos)
    { __api6(SYSCALL_API_GUI_SCROLLBAR, (uint32_t)x, (uint32_t)y, (uint32_t)h, (uint32_t)total, (uint32_t)visible, (uint32_t)pos); }
static inline void popup_menu(int x, int y, const char** items, int count, int sel)
    { __api5(SYSCALL_API_GUI_POPUP_MENU, (uint32_t)x, (uint32_t)y, (uint32_t)items, (uint32_t)count, (uint32_t)sel); }
static inline void titlebar(int x, int y, int w, const char* title, int active)
    { __api5(SYSCALL_API_GUI_TITLEBAR, (uint32_t)x, (uint32_t)y, (uint32_t)w, (uint32_t)title, (uint32_t)active); }
// TUI помощники
static inline int center_x(const char* text) {
    return (SCREEN_COLS - slen(text)) / 2;
}

static inline void msgbox(const char* title, const char* message) {
    int w = slen(message) + 4;
    if (w < 30) w = 30;
    int h = 5;
    int x = (SCREEN_COLS - w) / 2;
    int y = (SCREEN_ROWS - h) / 2;
    window(x, y, w, h, title);
    gotoxy(x + 2, y + 2);
    color(WHITE, BLUE);
    print(message);
    button(x + (w - 8) / 2, y + h - 2, "OK", 1);
    waitkey();
    clear();
}

// Системное
static inline void sysinfo(sys_info_t* info)       { __api1(SYSCALL_API_GET_SYSINFO, (uint32_t)info); }

static inline void get_username(char* buf) {
    sys_info_t info; sysinfo(&info); scopy(buf, info.username);
}
static inline void get_hostname(char* buf) {
    sys_info_t info; sysinfo(&info); scopy(buf, info.hostname);
}
static inline void get_cwd(char* buf) {
    sys_info_t info; sysinfo(&info); scopy(buf, info.cwd);
}
static inline void get_os_name(char* buf) {
    sys_info_t info; sysinfo(&info); scopy(buf, info.os_name);
}
static inline uint32_t get_mem_kb(void) {
    sys_info_t info; sysinfo(&info); return info.mem_kb;
}
static inline void set_username(const char* name)  { __api1(SYSCALL_API_SET_USERNAME, (uint32_t)name); }
static inline void set_hostname(const char* name)  { __api1(SYSCALL_API_SET_HOSTNAME, (uint32_t)name); }
static inline void get_time(uint8_t* h, uint8_t* m, uint8_t* s) {
    __api3(SYSCALL_API_GET_TIME, (uint32_t)h, (uint32_t)m, (uint32_t)s);
}
static inline void get_date(uint8_t* d, uint8_t* m, uint16_t* y) {
    __api3(SYSCALL_API_GET_DATE, (uint32_t)d, (uint32_t)m, (uint32_t)y);
}
static inline int str_compare(const char* a, const char* b) {
    return (int)__api2(SYSCALL_API_STR_COMPARE, (uint32_t)a, (uint32_t)b);
}
static inline int str_ncompare(const char* a, const char* b, int n) {
    return (int)__api3(SYSCALL_API_STR_NCOMPARE, (uint32_t)a, (uint32_t)b, (uint32_t)n);
}
static inline void str_copy(char* d, const char* s) {
    __api2(SYSCALL_API_STR_COPY, (uint32_t)d, (uint32_t)s);
}

#endif
