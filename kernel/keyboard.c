#include "keyboard.h"
#include "io.h"

// Буфер
static char key_buffer[KEYBOARD_BUFFER_SIZE];
static volatile int buffer_head = 0;
static volatile int buffer_tail = 0;

// Модификаторы
static volatile int shift_pressed = 0;
static volatile int ctrl_pressed = 0;
static volatile int caps_lock = 0;

// Символы
static const char sc_ascii_lower[128] = {
/*00*/  0,  27, '1','2','3','4','5','6','7','8','9','0','-','=','\b',
/*0F*/  '\t','q','w','e','r','t','y','u','i','o','p','[',']','\n',
/*1D*/  0,  'a','s','d','f','g','h','j','k','l',';','\'','`',
/*2A*/  0,  '\\','z','x','c','v','b','n','m',',','.','/',  0,
/*37*/  '*', 0,  ' '
};

// Символы верхнего регистра
static const char sc_ascii_upper[128] = {
/*00*/  0,  27, '!','@','#','$','%','^','&','*','(',')','_','+','\b',
/*0F*/  '\t','Q','W','E','R','T','Y','U','I','O','P','{','}','\n',
/*1D*/  0,  'A','S','D','F','G','H','J','K','L',':','"','~',
/*2A*/  0,  '|','Z','X','C','V','B','N','M','<','>','?',  0,
/*37*/  '*', 0,  ' '
};

// Добавление в буфер
static void buffer_put(char c) {
    int next = (buffer_head + 1) % KEYBOARD_BUFFER_SIZE;
    if (next != buffer_tail) {
        key_buffer[buffer_head] = c;
        buffer_head = next;
    }
}

// Инициализация
void keyboard_init(void) {
    buffer_head = 0;
    buffer_tail = 0;
    shift_pressed = 0;
    ctrl_pressed = 0;
    caps_lock = 0;
}

// Обработчик прерывания
void keyboard_callback(uint32_t scancode) {
    uint8_t code = (uint8_t)(scancode & 0xFF);
    int released = (code & 0x80) != 0;
    code &= 0x7F;  // Убираем бит отпускания
    
    // Модификаторы
    if (code == SC_LSHIFT || code == SC_RSHIFT) {
        shift_pressed = !released;
        return;
    }
    if (code == SC_LCTRL) {
        ctrl_pressed = !released;
        return;
    }
    if (code == SC_CAPSLOCK) {
        if (!released) caps_lock = !caps_lock;
        return;
    }
    
    // Игнорируем отпускание
    if (released) return;
    
    // Стрелки
    if (code == SC_ARROW_UP)    { buffer_put((char)KEY_ARROW_UP);    return; }
    if (code == SC_ARROW_DOWN)  { buffer_put((char)KEY_ARROW_DOWN);  return; }
    if (code == SC_ARROW_LEFT)  { buffer_put((char)KEY_ARROW_LEFT);  return; }
    if (code == SC_ARROW_RIGHT) { buffer_put((char)KEY_ARROW_RIGHT); return; }
    
    // Обычные клавиши
    if (code < 58) {  // Только валидные скан коды
        int use_shift = shift_pressed;
        
        // Caps Lock
        if (caps_lock && code >= 0x10 && code <= 0x32) {
            use_shift = !use_shift;
        }
        
        char c = use_shift ? sc_ascii_upper[code] : sc_ascii_lower[code];
        
        if (c != 0) {
            buffer_put(c);
        }
    }
}

// Есть ли символ?
int keyboard_haschar(void) {
    return buffer_head != buffer_tail;
}

// Взять без ожидания
char keyboard_getchar(void) {
    if (buffer_head == buffer_tail) return 0;
    char c = key_buffer[buffer_tail];
    buffer_tail = (buffer_tail + 1) % KEYBOARD_BUFFER_SIZE;
    return c;
}

// Ожидание символа
char keyboard_wait_char(void) {
    while (!keyboard_haschar()) {
        __asm__ volatile("hlt");
    }
    return keyboard_getchar();
}

// Очистка буфера
void keyboard_clear_buffer(void) {
    buffer_head = 0;
    buffer_tail = 0;
}

int keyboard_shift_pressed(void) { return shift_pressed; }
int keyboard_ctrl_pressed(void) { return ctrl_pressed; }