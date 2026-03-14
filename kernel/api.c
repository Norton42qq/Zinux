#include "api.h"
#include "vga.h"
#include "keyboard.h"
#include "string.h"
#include "io.h"
#include "fat16.h"
#include "ata.h"
#include "shell.h"
#include "system.h"

// Текст
static void ap_print(const char* s)    { vga_puts(s); }
static void ap_println(const char* s)  { vga_puts(s); vga_putchar('\n'); }
static void ap_print_char(char c)      { vga_putchar(c); }
static void ap_print_hex(uint32_t v)   { vga_put_hex(v); }
static void ap_print_int(int v) {
    if(v<0){ vga_putchar('-'); v=-v; }
    vga_put_dec((uint32_t)v);
}
static void ap_set_color(uint8_t fg, uint8_t bg) {
    vga_set_color((vga_color_t)fg, (vga_color_t)bg);
}

// Ввод
static char ap_read_char(void) { return keyboard_wait_char(); }
static void ap_read_line(char* buf, int max_len) {
    int pos = 0;
    while(pos < max_len - 1) {
        char c = keyboard_wait_char();
        if(c == '\n') { vga_putchar('\n'); break; }
        if(c == '\b') {
            if(pos > 0) { pos--; vga_backspace(); }
            continue;
        }
        if((unsigned char)c >= 32) {
            buf[pos++] = c;
            vga_putchar(c);
        }
    }
    buf[pos] = '\0';
}

// Экран
static void ap_clear(void)          { vga_clear(); }
static void ap_set_cur(int x, int y){ vga_set_cursor(x, y); }
static void ap_get_cur(int* x, int* y){ vga_get_cursor(x, y); }

// TUI отрисовка элементов
static void ap_draw_box(int x, int y, int w, int h, uint8_t fg, uint8_t bg) {
    vga_set_color((vga_color_t)fg, (vga_color_t)bg);
    
    // Углы
    vga_set_cursor(x, y);
    vga_putchar('\xDA');  // ┌ 
    vga_set_cursor(x + w - 1, y);
    vga_putchar('\xBF');  // ┐
    vga_set_cursor(x, y + h - 1);
    vga_putchar('\xC0');  // └
    vga_set_cursor(x + w - 1, y + h - 1);
    vga_putchar('\xD9');  // ┘
    
    // Горизонтальные линии
    for(int i = 1; i < w - 1; i++) {
        vga_set_cursor(x + i, y);
        vga_putchar('\xC4');  // ─
        vga_set_cursor(x + i, y + h - 1);
        vga_putchar('\xC4');
    }
    
    // Вертикальные линии
    for(int i = 1; i < h - 1; i++) {
        vga_set_cursor(x, y + i);
        vga_putchar('\xB3');  // │
        vga_set_cursor(x + w - 1, y + i);
        vga_putchar('\xB3');
    }
}

static void ap_draw_box_titled(int x, int y, int w, int h, const char* title, uint8_t fg, uint8_t bg) {
    ap_draw_box(x, y, w, h, fg, bg);
    
    // Заголовок приложения
    if(title && title[0]) {
        int title_len = strlen(title);
        int title_x = x + (w - title_len - 2) / 2;
        
        vga_set_cursor(title_x, y);
        vga_set_color((vga_color_t)fg, (vga_color_t)bg);
        vga_putchar(' ');
        vga_puts(title);
        vga_putchar(' ');
    }
}

static void ap_fill_rect(int x, int y, int w, int h, char ch, uint8_t fg, uint8_t bg) {
    vga_set_color((vga_color_t)fg, (vga_color_t)bg);
    for(int row = 0; row < h; row++) {
        vga_set_cursor(x, y + row);
        for(int col = 0; col < w; col++) {
            vga_putchar(ch);
        }
    }
}

static void ap_draw_hline(int x, int y, int len, uint8_t fg, uint8_t bg) {
    vga_set_color((vga_color_t)fg, (vga_color_t)bg);
    vga_set_cursor(x, y);
    for(int i = 0; i < len; i++) {
        vga_putchar('\xC4');  // ─
    }
}

static void ap_draw_vline(int x, int y, int len, uint8_t fg, uint8_t bg) {
    vga_set_color((vga_color_t)fg, (vga_color_t)bg);
    for(int i = 0; i < len; i++) {
        vga_set_cursor(x, y + i);
        vga_putchar('\xB3');  // │
    }
}

// TUI
static void ap_draw_window(int x, int y, int w, int h, const char* title) {
    // Тень
    ap_fill_rect(x + 2, y + 1, w, h, '\xB0', 8, 0);
    
    // Окно
    ap_fill_rect(x, y, w, h, ' ', 15, 1);
    ap_draw_box_titled(x, y, w, h, title, 15, 1);
}

static void ap_draw_button(int x, int y, const char* text, int selected) {
    uint8_t fg = selected ? 0 : 15;
    uint8_t bg = selected ? 15 : 1;
    
    vga_set_color((vga_color_t)fg, (vga_color_t)bg);
    vga_set_cursor(x, y);
    vga_puts("[ ");
    vga_puts(text);
    vga_puts(" ]");
}

static void ap_draw_input(int x, int y, int w, const char* text, int active) {
    uint8_t fg = active ? 0 : 7;
    uint8_t bg = active ? 7 : 0;
    
    vga_set_color((vga_color_t)fg, (vga_color_t)bg);
    vga_set_cursor(x, y);
    
    int text_len = strlen(text);
    vga_puts(text);
    // Заполнение пустоты
    for(int i = text_len; i < w; i++) {
        vga_putchar(' ');
    }
    
    // Курсор если активен
    if(active) {
        vga_set_cursor(x + text_len, y);
    }
}

static void ap_draw_progress(int x, int y, int w, int percent) {
    vga_set_cursor(x, y);
    vga_set_color(7, 0);
    vga_putchar('[');
    
    int filled = (w - 2) * percent / 100;
    for(int i = 0; i < w - 2; i++) {
        if(i < filled) {
            vga_set_color(15, 2);
            vga_putchar('\xDB');  // █
        } else {
            vga_set_color(8, 0);
            vga_putchar('\xB0');  // ░
        }
    }
    
    vga_set_color(7, 0);
    vga_putchar(']');
    vga_putchar(' ');
    vga_put_dec(percent);
    vga_putchar('%');
}

// Строки
static int  ap_strcmp(const char* a, const char* b) { return strcmp(a, b); }
static int  ap_strlen(const char* s) { return strlen(s); }
static void ap_strcpy(char* d, const char* s) { strcpy(d, s); }
static void ap_memcpy(void* d, const void* s, uint32_t n) { memcpy(d, s, n); }
static void ap_memset(void* d, uint8_t v, uint32_t n) { memset(d, v, n); }

// Время
static void ap_get_time(uint8_t* h, uint8_t* m, uint8_t* s) {
    outb(0x70, 0x04); uint8_t bh = inb(0x71);
    outb(0x70, 0x02); uint8_t bm = inb(0x71);
    outb(0x70, 0x00); uint8_t bs = inb(0x71);
    if(h) *h = ((bh >> 4) * 10) + (bh & 0xF);
    if(m) *m = ((bm >> 4) * 10) + (bm & 0xF);
    if(s) *s = ((bs >> 4) * 10) + (bs & 0xF);
}

static void ap_get_date(uint8_t* d, uint8_t* m, uint16_t* y) {
    outb(0x70, 0x07); uint8_t bd = inb(0x71);
    outb(0x70, 0x08); uint8_t bm = inb(0x71);
    outb(0x70, 0x09); uint8_t by = inb(0x71);
    if(d) *d = ((bd >> 4) * 10) + (bd & 0xF);
    if(m) *m = ((bm >> 4) * 10) + (bm & 0xF);
    if(y) *y = 2000 + ((by >> 4) * 10) + (by & 0xF);
}

static void ap_delay(uint32_t ms) {
    for(volatile uint32_t i = 0; i < ms * 1000; i++) {
        __asm__ volatile("nop");
    }
}

// Файлы
static int      ap_fexists(const char* f) { return fat16_file_exists(f); }
static int      ap_fread(const char* f, void* b, uint32_t m) { return fat16_read_file(f, b, m); }
static uint32_t ap_fsize(const char* f) { return fat16_file_size(f); }
static int      ap_fwrite(const char* f, const void* b, uint32_t n) {
    int r = fat16_write_file(f, b, n);
    if (r > 0) ata_flush();
    return r;
}

// Клавиатура
static int  ap_haskey(void)    { return keyboard_haschar(); }
static char ap_getkey(void)    { return keyboard_getchar(); }
static void ap_clearkeys(void) { keyboard_clear_buffer(); }

// Системная информация
static void ap_get_sysinfo(sys_info_t* info) {
    if (!info) return;
    strcpy(info->username,   current_config.username);
    strcpy(info->hostname,   current_config.hostname);
    strcpy(info->os_name,    "Zinux");
    strcpy(info->os_version, "0.1");
    extern const char* shell_get_cwd(void);
    strcpy(info->cwd, shell_get_cwd());
    system_info_t* si = system_get_info();
    info->mem_kb = si->mem_lower + si->mem_upper;
}

static void ap_set_username(const char* name) {
    if (!name) return;
    strncpy(current_config.username, name, 31);
    current_config.username[31] = '\0';
    shell_save_config();
}

static void ap_set_hostname(const char* name) {
    if (!name) return;
    strncpy(current_config.hostname, name, 31);
    current_config.hostname[31] = '\0';
    shell_save_config();
}

// Таблица команд
void api_setup_table(void) {
    api_table_t* t = (api_table_t*)API_TABLE_ADDRESS;
    
    t->print       = ap_print;
    t->println     = ap_println;
    t->print_char  = ap_print_char;
    t->print_int   = ap_print_int;
    t->print_hex   = ap_print_hex;
    t->set_color   = ap_set_color;
    
    t->read_char   = ap_read_char;
    t->read_line   = ap_read_line;
    
    t->clear_screen = ap_clear;
    t->set_cursor  = ap_set_cur;
    t->get_cursor  = ap_get_cur;
    
    t->draw_box        = ap_draw_box;
    t->draw_box_titled = ap_draw_box_titled;
    t->fill_rect       = ap_fill_rect;
    t->draw_hline      = ap_draw_hline;
    t->draw_vline      = ap_draw_vline;
    
    t->draw_window   = ap_draw_window;
    t->draw_button   = ap_draw_button;
    t->draw_input    = ap_draw_input;
    t->draw_progress = ap_draw_progress;
    
    t->str_compare = ap_strcmp;
    t->str_length  = ap_strlen;
    t->str_copy    = ap_strcpy;
    t->mem_copy    = ap_memcpy;
    t->mem_set     = ap_memset;
    
    t->get_time = ap_get_time;
    t->get_date = ap_get_date;
    t->delay    = ap_delay;
    
    t->file_exists = ap_fexists;
    t->file_read   = ap_fread;
    t->file_size   = ap_fsize;
    t->file_write  = ap_fwrite;
    
    t->has_key    = ap_haskey;
    t->get_key    = ap_getkey;
    t->clear_keys = ap_clearkeys;

    t->get_sysinfo  = ap_get_sysinfo;
    t->set_username = ap_set_username;
    t->set_hostname = ap_set_hostname;
}

void api_init(void) {
    api_setup_table();
}