#ifndef MOUSE_H
#define MOUSE_H

#include <stdint.h>

#define MOUSE_BTN_LEFT   (1 << 0)
#define MOUSE_BTN_RIGHT  (1 << 1)
#define MOUSE_BTN_MIDDLE (1 << 2)

typedef enum {
    MOUSE_EV_MOVE    = 0,   // курсор перемещён
    MOUSE_EV_PRESS   = 1,   // кнопка нажата
    MOUSE_EV_RELEASE = 2,   // кнопка отпущена
    MOUSE_EV_SCROLL  = 3,   // прокрутка колёсиком
} MouseEventType;

// Cостояние мыши
typedef struct {
    int     x;          // абсолютная X позиция курсора
    int     y;          // абсолютная Y позиция курсора
    uint8_t buttons;    // битовая маска зажатых кнопок
    int     wheel;      // смещение колёсика за последний пакет
} MouseState;

// Событие мыши
typedef struct {
    MouseEventType type;
    int     x, y;      // позиция в момент события
    int     dx, dy;    // смещение за этот пакет
    int     wheel;
    uint8_t buttons;
} MouseEvent;

// Тип функции-обработчика событий
typedef void (*mouse_event_fn)(const MouseEvent *ev);

// API

void mouse_init(int screen_w, int screen_h);

MouseState mouse_get_state(void);

void mouse_set_callback(mouse_event_fn fn);

void mouse_set_sensitivity(int sx, int sy);

void mouse_set_pos(int x, int y);

void mouse_set_bounds(int w, int h);
int  mouse_is_ready(void);

// Обработчик IRQ12
struct regs;
void mouse_handler(struct regs *r);

#endif