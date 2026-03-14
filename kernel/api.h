#ifndef API_H
#define API_H

#include <stdint.h>

#define API_TABLE_ADDRESS  0x100000

// Системная информация
typedef struct {
    char     username[32];
    char     hostname[32];
    char     os_name[32];
    char     os_version[16];
    char     cwd[64]; // текущая директория
    uint32_t mem_kb;
} __attribute__((packed)) sys_info_t;

typedef struct {
    // Текст
    void     (*print)      (const char* str);
    void     (*println)    (const char* str);
    void     (*print_char) (char c);
    void     (*print_int)  (int value);
    void     (*print_hex)  (uint32_t value);
    void     (*set_color)  (uint8_t fg, uint8_t bg);
    
    // Ввод
    char     (*read_char)  (void);
    void     (*read_line)  (char* buf, int max_len);
    
    // Экран
    void     (*clear_screen)(void);
    void     (*set_cursor)  (int x, int y);
    void     (*get_cursor)  (int* x, int* y);
    
    // TUI рамка
    void     (*draw_box)    (int x, int y, int w, int h, uint8_t fg, uint8_t bg);
    void     (*draw_box_titled)(int x, int y, int w, int h, const char* title, uint8_t fg, uint8_t bg);
    void     (*fill_rect)   (int x, int y, int w, int h, char ch, uint8_t fg, uint8_t bg);
    void     (*draw_hline)  (int x, int y, int len, uint8_t fg, uint8_t bg);
    void     (*draw_vline)  (int x, int y, int len, uint8_t fg, uint8_t bg);
    
    // TUI окно
    void     (*draw_window) (int x, int y, int w, int h, const char* title);
    void     (*draw_button) (int x, int y, const char* text, int selected);
    void     (*draw_input)  (int x, int y, int w, const char* text, int active);
    void     (*draw_progress)(int x, int y, int w, int percent);
    
    // Строки
    int      (*str_compare) (const char* s1, const char* s2);
    int      (*str_length)  (const char* str);
    void     (*str_copy)    (char* dest, const char* src);
    void     (*mem_copy)    (void* dest, const void* src, uint32_t size);
    void     (*mem_set)     (void* dest, uint8_t val, uint32_t size);
    
    // Время
    void     (*get_time)    (uint8_t* h, uint8_t* m, uint8_t* s);
    void     (*get_date)    (uint8_t* d, uint8_t* m, uint16_t* y);
    void     (*delay)       (uint32_t ms);
    
    // Файлы
    int      (*file_exists) (const char* fn);
    int      (*file_read)   (const char* fn, void* buf, uint32_t max);
    uint32_t (*file_size)   (const char* fn);
    int      (*file_write)  (const char* fn, const void* buf, uint32_t size);
    
    // Клавиатура
    int      (*has_key)     (void);
    char     (*get_key)     (void);
    void     (*clear_keys)  (void);

    // Система
    void     (*get_sysinfo) (sys_info_t* info);
    void     (*set_username)(const char* name);
    void     (*set_hostname)(const char* name);
    
} __attribute__((packed)) api_table_t;

void api_init(void);
void api_setup_table(void);

#endif