/*
 * zxe_api.h - Zinux SDK v2.0 Clean + TUI
 * Text User Interface для .zxe программ
 */
#ifndef ZXE_API_H
#define ZXE_API_H

typedef unsigned char  uint8_t;
typedef unsigned short uint16_t;
typedef unsigned int   uint32_t;
#define NULL ((void*)0)

/* ═══ Клавиши ═══ */
#define KEY_UP        ((char)128)
#define KEY_DOWN      ((char)129)
#define KEY_LEFT      ((char)130)
#define KEY_RIGHT     ((char)131)
#define KEY_ESC       27
#define KEY_ENTER     '\n'
#define KEY_SPACE     ' '
#define KEY_BACKSPACE '\b'
#define KEY_TAB       '\t'

/* ═══ Цвета ═══ */
#define BLACK         0
#define BLUE          1
#define GREEN         2
#define CYAN          3
#define RED           4
#define MAGENTA       5
#define BROWN         6
#define LIGHT_GREY    7
#define DARK_GREY     8
#define LIGHT_BLUE    9
#define LIGHT_GREEN   10
#define LIGHT_CYAN    11
#define LIGHT_RED     12
#define LIGHT_MAGENTA 13
#define YELLOW        14
#define WHITE         15

/* ═══ Псевдографика ═══ */
#define CHAR_HLINE    '\xC4'  /* ─ */
#define CHAR_VLINE    '\xB3'  /* │ */
#define CHAR_TL       '\xDA'  /* ┌ */
#define CHAR_TR       '\xBF'  /* ┐ */
#define CHAR_BL       '\xC0'  /* └ */
#define CHAR_BR       '\xD9'  /* ┘ */
#define CHAR_SHADE    '\xB0'  /* ░ */
#define CHAR_BLOCK    '\xDB'  /* █ */

/* ═══ Экран ═══ */
#define SCREEN_COLS  80
#define SCREEN_ROWS  30

/* ═══════════════════════════════════════════════════ */
/* API ТАБЛИЦА */
/* ═══════════════════════════════════════════════════ */

#define API_TABLE_ADDRESS 0x100000

/* Системная информация */
typedef struct {
    char     username[32];
    char     hostname[32];
    char     os_name[32];
    char     os_version[16];
    char     cwd[64];
    uint32_t mem_kb;
} __attribute__((packed)) sys_info_t;

typedef struct {
    void     (*print)      (const char* str);
    void     (*println)    (const char* str);
    void     (*print_char) (char c);
    void     (*print_int)  (int value);
    void     (*print_hex)  (uint32_t value);
    void     (*set_color)  (uint8_t fg, uint8_t bg);
    char     (*read_char)  (void);
    void     (*read_line)  (char* buf, int max_len);
    void     (*clear_screen)(void);
    void     (*set_cursor)  (int x, int y);
    void     (*get_cursor)  (int* x, int* y);
    void     (*draw_box)    (int x, int y, int w, int h, uint8_t fg, uint8_t bg);
    void     (*draw_box_titled)(int x, int y, int w, int h, const char* title, uint8_t fg, uint8_t bg);
    void     (*fill_rect)   (int x, int y, int w, int h, char ch, uint8_t fg, uint8_t bg);
    void     (*draw_hline)  (int x, int y, int len, uint8_t fg, uint8_t bg);
    void     (*draw_vline)  (int x, int y, int len, uint8_t fg, uint8_t bg);
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
    /* Система */
    void     (*get_sysinfo) (sys_info_t* info);
    void     (*set_username)(const char* name);
    void     (*set_hostname)(const char* name);
} __attribute__((packed)) api_table_t;

static api_table_t* const api = (api_table_t*)API_TABLE_ADDRESS;

/* ═══════════════════════════════════════════════════ */
/* УДОБНЫЕ ОБЁРТКИ */
/* ═══════════════════════════════════════════════════ */

/* Текст */
static inline void print(const char* s)      { api->print(s); }
static inline void println(const char* s)    { api->println(s); }
static inline void putchar(char c)           { api->print_char(c); }
static inline void print_int(int v)          { api->print_int(v); }
static inline void print_hex(uint32_t v)     { api->print_hex(v); }
static inline void color(uint8_t f, uint8_t b) { api->set_color(f, b); }

/* Ввод */
static inline char waitkey(void)             { return api->read_char(); }
static inline void input(char* b, int m)     { api->read_line(b, m); }
static inline int  kbhit(void)               { return api->has_key(); }
static inline char getch(void)               { return api->get_key(); }
static inline void flushkeys(void)           { api->clear_keys(); }

/* Экран */
static inline void clear(void)               { api->clear_screen(); }
static inline void gotoxy(int x, int y)      { api->set_cursor(x, y); }
static inline void getxy(int* x, int* y)     { api->get_cursor(x, y); }

/* TUI - Рисование */
static inline void box(int x, int y, int w, int h, uint8_t fg, uint8_t bg) {
    api->draw_box(x, y, w, h, fg, bg);
}
static inline void box_titled(int x, int y, int w, int h, const char* t, uint8_t fg, uint8_t bg) {
    api->draw_box_titled(x, y, w, h, t, fg, bg);
}
static inline void fill(int x, int y, int w, int h, char c, uint8_t fg, uint8_t bg) {
    api->fill_rect(x, y, w, h, c, fg, bg);
}
static inline void hline(int x, int y, int len, uint8_t fg, uint8_t bg) {
    api->draw_hline(x, y, len, fg, bg);
}
static inline void vline(int x, int y, int len, uint8_t fg, uint8_t bg) {
    api->draw_vline(x, y, len, fg, bg);
}

/* TUI - Компоненты */
static inline void window(int x, int y, int w, int h, const char* t) {
    api->draw_window(x, y, w, h, t);
}
static inline void button(int x, int y, const char* t, int sel) {
    api->draw_button(x, y, t, sel);
}
static inline void textbox(int x, int y, int w, const char* t, int act) {
    api->draw_input(x, y, w, t, act);
}
static inline void progress(int x, int y, int w, int pct) {
    api->draw_progress(x, y, w, pct);
}

/* Строки */
static inline int  streq(const char* a, const char* b) { return api->str_compare(a, b) == 0; }
static inline int  slen(const char* s)       { return api->str_length(s); }
static inline void scopy(char* d, const char* s) { api->str_copy(d, s); }

/* Файлы */
static inline int      fexists(const char* f) { return api->file_exists(f); }
static inline int      fread(const char* f, void* b, uint32_t m) { return api->file_read(f, b, m); }
static inline uint32_t fsize(const char* f)   { return api->file_size(f); }
static inline int      fwrite(const char* f, const void* b, uint32_t n) { return api->file_write(f, b, n); }

/* Время */
static inline void delay(uint32_t ms)         { api->delay(ms); }

/* ═══════════════════════════════════════════════════ */
/* TUI ХЕЛПЕРЫ */
/* ═══════════════════════════════════════════════════ */

/* Центрировать текст */
static inline int center_x(const char* text) {
    return (SCREEN_COLS - slen(text)) / 2;
}

/* Показать сообщение в центре */
static inline void msgbox(const char* title, const char* message) {
    int w = slen(message) + 4;
    if(w < 30) w = 30;
    int h = 5;
    int x = (SCREEN_COLS - w) / 2;
    int y = (SCREEN_ROWS - h) / 2;
    
    window(x, y, w, h, title);
    gotoxy(x + 2, y + 2);
    color(WHITE, BLUE);
    print(message);
    
    gotoxy(x + (w - 8) / 2, y + h - 2);
    button(x + (w - 8) / 2, y + h - 2, "OK", 1);
    
    waitkey();
    clear();
}

/* ═══════════════════════════════════════════════════ */
/* СИСТЕМА */
/* ═══════════════════════════════════════════════════ */

/* Получить всю системную информацию одним вызовом */
static inline void sysinfo(sys_info_t* info)       { api->get_sysinfo(info); }

/* Быстрые геттеры — копируют строку в буфер предоставленный вызывающим */
static inline void get_username(char* buf) {
    sys_info_t info;
    api->get_sysinfo(&info);
    api->str_copy(buf, info.username);
}

static inline void get_hostname(char* buf) {
    sys_info_t info;
    api->get_sysinfo(&info);
    api->str_copy(buf, info.hostname);
}

static inline void get_cwd(char* buf) {
    sys_info_t info;
    api->get_sysinfo(&info);
    api->str_copy(buf, info.cwd);
}

static inline void get_os_name(char* buf) {
    sys_info_t info;
    api->get_sysinfo(&info);
    api->str_copy(buf, info.os_name);
}

static inline uint32_t get_mem_kb(void) {
    sys_info_t info;
    api->get_sysinfo(&info);
    return info.mem_kb;
}

/* Сеттеры — меняют значение и сохраняют конфиг на диск */
static inline void set_username(const char* name)  { api->set_username(name); }
static inline void set_hostname(const char* name)  { api->set_hostname(name); }

#endif