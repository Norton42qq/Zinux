#include <stddef.h>
#include "mouse.h"
#include "../io.h"

// Порты
#define PS2_DATA    0x60
#define PS2_STATUS  0x64   // чтение 
#define PS2_CMD     0x64   // запись 

// Биты статусного регистра
#define ST_OBF   (1 << 0)  // Output Buffer Full - байт для чтения
#define ST_IBF   (1 << 1)  // Input  Buffer Full - контроллер занят     
#define ST_AUX   (1 << 5)  // байт от мыши (1) или клавиатуры (0)       

// Команды контроллера i8042
#define CTL_READ_CFG    0x20
#define CTL_WRITE_CFG   0x60
#define CTL_ENABLE_AUX  0xA8
#define CTL_TEST_AUX    0xA9
#define CTL_WRITE_AUX   0xD4

// Команды
#define M_RESET    0xFF
#define M_ENABLE   0xF4
#define M_DISABLE  0xF5
#define M_SET_RATE 0xF3
#define M_GET_ID   0xF2
#define M_SET_RES  0xE8
#define M_ACK      0xFA
#define M_RESEND   0xFE

// Биты первого байта пакета
#define P0_LEFT    (1 << 0)
#define P0_RIGHT   (1 << 1)
#define P0_MIDDLE  (1 << 2)
#define P0_ALWAYS1 (1 << 3)
#define P0_XOVER   (1 << 6)
#define P0_YOVER   (1 << 7)

// Состояние драйвера
static MouseState     g_mouse;
static mouse_event_fn g_cb;

static uint8_t pkt[4];
static int     pkt_pos;
static int     pkt_len;
static int     ready;

static int max_x, max_y;
static int sens_x = 1, sens_y = 1;

// Низкоуровневые I/O

static int wait_write(void) {
    int n = 100000;
    while (--n && (inb(PS2_STATUS) & ST_IBF)) io_wait();
    return n > 0;
}

static int wait_read(void) {
    int n = 100000;
    while (--n && !(inb(PS2_STATUS) & ST_OBF)) io_wait();
    return n > 0;
}

static void flush(void) {
    int n = 32;
    while (n-- && (inb(PS2_STATUS) & ST_OBF)) { io_wait(); inb(PS2_DATA); io_wait(); }
}

static int ctl_cmd(uint8_t c)  {
    if (!wait_write()) return 0;
    outb(PS2_CMD, c);
    io_wait();
    return 1;
}
static int ctl_data(uint8_t d) {
    if (!wait_write()) return 0;
    outb(PS2_DATA, d);
    io_wait();
    return 1;
}
static int read_byte(uint8_t* out)  {
    if (!wait_read()) return 0;
    *out = inb(PS2_DATA);
    return 1;
}

static int mouse_cmd(uint8_t cmd) {
    int r = 3;
    uint8_t a;
    while (r--) {
        if (!ctl_cmd(CTL_WRITE_AUX) || !ctl_data(cmd)) return 0;
        if (!read_byte(&a)) return 0;
        if (a == M_ACK)    return 1;
        if (a != M_RESEND) return 0;
    }
    return 0;
}

static int mouse_cmd2(uint8_t c, uint8_t a) { return mouse_cmd(c) && mouse_cmd(a); }

static void irq12_mask(void)   { outb(0xA1, inb(0xA1) |  (1 << 4)); }
static void irq12_unmask(void) { outb(0xA1, inb(0xA1) & ~(1 << 4)); }
static void irq2_unmask(void)  { outb(0x21, inb(0x21) & ~(1 << 2)); } // каскад slave PIC

// Инициализация

void mouse_init(int screen_w, int screen_h) {
    uint8_t cfg = 0;

    max_x = screen_w - 1;  max_y = screen_h - 1;
    g_mouse.x = screen_w / 2;  g_mouse.y = screen_h / 2;
    g_mouse.buttons = 0;  g_mouse.wheel = 0;
    pkt_pos = 0;  pkt_len = 3;  ready = 0;  g_cb = NULL;

    irq12_mask();
    flush();

    // Шаг 1: включить AUX порт
    if (!ctl_cmd(CTL_ENABLE_AUX)) goto fail;
    flush();

    // Шаг 2: считать конфиг, выключить AUX IRQ на время init
    if (!ctl_cmd(CTL_READ_CFG) || !read_byte(&cfg)) goto fail;
    cfg &= ~(1 << 1);  // AUX IRQ off 
    cfg &= ~(1 << 5);  // AUX clock on (бит5=1 = disable, поэтому сбрасываем)
    if (!ctl_cmd(CTL_WRITE_CFG) || !ctl_data(cfg)) goto fail;

    // Шаг 3: тест AUX - 0x00 = OK
    uint8_t tr = 0xFF;
    if (!ctl_cmd(CTL_TEST_AUX) || !read_byte(&tr)) goto fail;
    if (tr != 0x00) goto fail;

    // Шаг 4: сброс мыши
    if (!mouse_cmd(M_RESET)) goto fail;
    {
        uint8_t b;
        if (!read_byte(&b) || b != 0xAA) goto fail;
        if (!read_byte(&b)) goto fail;  // обычно 0x00
    }
    flush();

    // Шаг 5: детект IntelliMouse
    mouse_cmd2(M_SET_RATE, 200);
    mouse_cmd2(M_SET_RATE, 100);
    mouse_cmd2(M_SET_RATE,  80);
    if (!mouse_cmd(M_GET_ID)) goto fail;
    {
        uint8_t id = 0;
        if (!read_byte(&id)) goto fail;
        (void)id;
        pkt_len = 3;
    }
    flush();

    // Шаг 6: чувствительность
    mouse_cmd2(M_SET_RES,  3);
    mouse_cmd2(M_SET_RATE, 100);

    // Шаг 7: включить AUX IRQ ДО ENABLE - не перечитывать cfg, используем уже известный
    cfg |= (1 << 1);   // AUX IRQ on
    cfg &= ~(1 << 5);  // clock on  
    if (!ctl_cmd(CTL_WRITE_CFG) || !ctl_data(cfg)) goto fail;

    // Шаг 8: разрешить мыши слать пакеты
    if (!mouse_cmd(M_ENABLE)) goto fail;
    flush();

    ready = 1;
    irq2_unmask();
    irq12_unmask();
    return;

fail:
    ready = 0;
    pkt_pos = 0;
    mouse_cmd(M_DISABLE);
    flush();
    cfg &= ~(1 << 1);
    if (ctl_cmd(CTL_WRITE_CFG)) (void)ctl_data(cfg);
    irq12_mask();
}

// API

MouseState mouse_get_state(void) { return g_mouse; }
void mouse_set_callback(mouse_event_fn fn) { g_cb = fn; }

static void clamp(void) {
    if (g_mouse.x < 0)     g_mouse.x = 0;
    if (g_mouse.y < 0)     g_mouse.y = 0;
    if (g_mouse.x > max_x) g_mouse.x = max_x;
    if (g_mouse.y > max_y) g_mouse.y = max_y;
}

// Обработчик IRQ12
void mouse_handler(struct regs *r) {
    (void)r;

    uint8_t st = inb(PS2_STATUS);

    if (!(st & ST_OBF)) return;       // буфер пуст

    uint8_t byte = inb(PS2_DATA);     // читаем ВСЕГДА

    if (!ready) {
        pkt_pos = 0;
        return;
    }

    // Синхронизация пакета
    if (pkt_pos == 0 && !(byte & P0_ALWAYS1)) return;
    if (pkt_pos > 0 && (byte & P0_ALWAYS1)) {
        pkt[0] = byte;
        pkt_pos = 1;
        return;
    }

    pkt[pkt_pos++] = byte;
    if (pkt_pos < pkt_len) return;  // ожидания остатка пакета

    pkt_pos = 0;  // сброс позиции

    uint8_t flags = pkt[0];
    if (flags & (P0_XOVER | P0_YOVER)) return;  // overflow - мусор

    // Кнопки
    uint8_t old_btn = g_mouse.buttons;
    g_mouse.buttons = 0;
    if (flags & P0_LEFT)   g_mouse.buttons |= MOUSE_BTN_LEFT;
    if (flags & P0_RIGHT)  g_mouse.buttons |= MOUSE_BTN_RIGHT;
    if (flags & P0_MIDDLE) g_mouse.buttons |= MOUSE_BTN_MIDDLE;

    // Движение
    int dx = (int)(int8_t)pkt[1];
    int dy = -(int)(int8_t)pkt[2];
    if (sens_x > 1) dx /= sens_x;
    if (sens_y > 1) dy /= sens_y;
    g_mouse.x += dx;
    g_mouse.y += dy;
    clamp();

    // Колесо
    g_mouse.wheel = 0;
    if (pkt_len == 4) {
        int8_t w = (int8_t)(pkt[3] & 0x0F);
        if (w & 0x08) w |= (int8_t)0xF0;
        g_mouse.wheel = (int)w;
    }

    if (!g_cb) return;

    uint8_t pressed  = (~old_btn) & g_mouse.buttons;
    uint8_t released =   old_btn  & (~g_mouse.buttons);
    if (!pressed && !released && !dx && !dy && !g_mouse.wheel) return;

    MouseEvent ev;
    ev.x = g_mouse.x;  ev.y = g_mouse.y;
    ev.dx = dx;  ev.dy = dy;
    ev.wheel = g_mouse.wheel;
    ev.buttons = g_mouse.buttons;

    if      (pressed & MOUSE_BTN_LEFT)  ev.type = MOUSE_EV_PRESS;
    else if (released & MOUSE_BTN_LEFT) ev.type = MOUSE_EV_RELEASE;
    else if (g_mouse.wheel)             ev.type = MOUSE_EV_SCROLL;
    else                                ev.type = MOUSE_EV_MOVE;

    g_cb(&ev);
}

// Утилиты

void mouse_set_sensitivity(int sx, int sy) {
    sens_x = (sx < 1) ? 1 : (sx > 8 ? 8 : sx);
    sens_y = (sy < 1) ? 1 : (sy > 8 ? 8 : sy);
}

void mouse_set_pos(int x, int y) { g_mouse.x = x; g_mouse.y = y; clamp(); }

void mouse_set_bounds(int w, int h) { max_x = w-1; max_y = h-1; clamp(); }
int mouse_is_ready(void) { return ready; }