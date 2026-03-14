#ifndef KEYBOARD_H
#define KEYBOARD_H

#include <stdint.h>

#define KEYBOARD_DATA_PORT      0x60
#define KEYBOARD_STATUS_PORT    0x64
#define KEYBOARD_BUFFER_SIZE    256

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

#define KEY_ARROW_UP    ((char)128)
#define KEY_ARROW_DOWN  ((char)129)
#define KEY_ARROW_LEFT  ((char)130)
#define KEY_ARROW_RIGHT ((char)131)

void keyboard_init(void);
void keyboard_callback(uint32_t scancode);
char keyboard_getchar(void);
int  keyboard_haschar(void);
char keyboard_wait_char(void);
void keyboard_clear_buffer(void);

int keyboard_shift_pressed(void);
int keyboard_ctrl_pressed(void);

#endif