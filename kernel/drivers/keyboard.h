#ifndef KEYBOARD_H
#define KEYBOARD_H

#include <stdint.h>

#define KEYBOARD_DATA_PORT      0x60
#define KEYBOARD_STATUS_PORT    0x64
#define KEYBOARD_BUFFER_SIZE    256

// Скан коды
#define SC_ESCAPE       0x01
#define SC_BACKSPACE    0x0E
#define SC_TAB          0x0F
#define SC_ENTER        0x1C
#define SC_LCTRL        0x1D
#define SC_LSHIFT       0x2A
#define SC_RSHIFT       0x36
#define SC_LALT         0x38
#define SC_CAPSLOCK     0x3A
#define SC_ARROW_UP     0x48
#define SC_ARROW_DOWN   0x50
#define SC_ARROW_LEFT   0x4B
#define SC_ARROW_RIGHT  0x4D

// Специальные коды
#define KEY_ARROW_UP    ((char)128)
#define KEY_ARROW_DOWN  ((char)129)
#define KEY_ARROW_LEFT  ((char)130)
#define KEY_ARROW_RIGHT ((char)131)

// Raw событие клавиши: scancode + флаг нажатия/отпускания
#define RAW_KEY_BUF_SIZE 64

typedef struct {
    uint8_t scancode; // скан код без бита 0x80
    uint8_t pressed;  // 1 = нажата, 0 = отпущена
} raw_key_event_t;

void keyboard_init(void);
void keyboard_callback(uint32_t scancode);

// ASCII интерфейс для shell
char keyboard_getchar(void);
int  keyboard_haschar(void);
char keyboard_wait_char(void);
void keyboard_clear_buffer(void);
int  keyboard_shift_pressed(void);
int  keyboard_ctrl_pressed(void);

// Raw интерфейс
int  keyboard_raw_has(void);
int  keyboard_raw_get(raw_key_event_t* ev);

#endif