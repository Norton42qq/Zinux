# Zinux - Документация по API & SDK
---

## Содержание

- [1. Введение](#1-введение)
- [2. Кольца привилегий](#2-кольца-привилегий)
- [3. Формат ZXE](#3-формат-zxe)
- [4. SDK - Создание программ](#4-sdk--создание-программ)
- [5. API Reference](#5-api-reference)
- [6. Системная информация](#6-системная-информация)
- [7. Пиксельная графика или GUI](#7-пиксельная-графика-или-gui)
- [8. Мышь](#8-мышь)
- [9. ID системных вызовов](#9-id-системных-вызовов)
- [10. Команды оболочки](#10-команды-оболочки)
- [11. Kernel Panic](#11-kernel-panic)
- [12. Кириллица и русский язык](#12-кириллица-и-русский-язык)
- [13. Известные ограничения и решения](#13-известные-ограничения-и-решения)

---

## 1. Введение

Zinux - Российская 32-х битная операционная система созданная школьником по заказу Роскомнадзор и лично В.В. Путиным. Программы компилируются в формат .ZXE.

Программа никогда не обращается к оборудованию напрямую. Весь доступ к ОС происходит через системный вызов `int 0x80`.

```
┌─────────────────────────────────────┐
│         .ZXE программа (Ring 2)     │
│  #include "zxe_api.h"               │
│  int main(int argc, char* argv[])   │
└──────────────┬──────────────────────┘
               │  int 0x80 (syscall)
┌──────────────▼──────────────────────┐
│   Ядро Zinux (Ring 0)               │
│   VGA · FAT16 · Keyboard · RTC ...  │
└─────────────────────────────────────┘
```

---

## 2. Кольца привилегий

### 2.1 Уровни привилегий

| Кольцо | Содержимое | Файлы |
|--------|-----------|-------|
| Ring 0 | Ядро, загрузчик, libc, шелл | `kernel/*`, `boot/*`, `libc/*` |
| Ring 1 | Драйверы | `kernel/drivers/*` |
| Ring 2 | Пользовательские программы (.ZXE) | `developmentfolder/*.c(.zxe)`, `programs/*.zxe` |

### 2.2 Механизм системного вызова

Программа Ring 2 вызывает ядро через `int 0x80`:

```
EAX = 1  -> putc    (вывод символа)
EAX = 2  -> exit    (завершение программы)
EAX = 3  -> api     (вызов функции по ID)
           EBX = ID функции
           ECX = указатель на аргументы
```

> **ВАЖНО:** Прямой вызов `api->функция()` через указатель `0x100000` работает только из Ring 0 (ядро). Из Ring 2 это вызовет **General Protection Fault (#13)**. Все функции в `zxe_api.h` - это обёртки над `int 0x80`, использовать можно только их.
---

## 3. Формат ZXE

### 3.1 Структура заголовка (48 байт)

```c
typedef struct {
    uint32_t magic;        // 0x00: 0x01455A58 ("ZXE\x01")
    uint8_t  version;      // 0x04: = 1
    uint8_t  flags;        // 0x05: зарезервировано, = 0
    uint16_t header_size;  // 0x06: = 48
    uint32_t code_size;    // 0x08: размер бинарного кода
    uint32_t data_size;    // 0x0C: = 0
    uint32_t bss_size;     // 0x10: = 0
    uint32_t entry_offset; // 0x14: смещение точки входа = 0
    char     name[16];     // 0x18: имя программы (null-terminated)
    char     reserved[8];  // 0x28: зарезервировано
} __attribute__((packed)) zxe_header_t;
```

### 3.2 Памятка

```
Адрес        Размер    Содержимое
──────────────────────────────────────────────────────────
0x00000      1 KB      IVT + BDA (BIOS)
0x07C00      512 B     Stage 1 (Базовый загрузчик)
0x07E00      4 KB      Stage 2 (Zrub Boot Loader)
0x08000      4 B       Адрес LFB от VESA (uint32)
0x08004      4 B       Флаг RAM диска (кол-во секторов)
0x08008      2 B       VESA pitch BytesPerScanLine
0x08009      1 B       Boot flags (0x01 = safe mode)
0x10000      256 KB    Ядро Zinux (Ring 0)
0x50000      192 KB    FAT16 RAM буфер (диск в памяти)
0x90000      ↓         Стек ядра (растёт вниз)
0xA0000                VGA/VESA область (BIOS)
0x100000     ~1 KB     API таблица (указатели функций)
0x200000     64 KB     Текущая ZXE программа (Ring 2)
0x2F0000     ↓         Стек пользовательской программы
```

> Программа не должна читать/писать адреса ниже `0x100000`. Попытка - General Protection Fault или некорректная работа ядра вплоть до Kernel panic.

### 3.3 Коды возврата загрузчика

```c
#define ZXE_OK         0   // успех
#define ZXE_ERROR     -1   // ошибка чтения с диска
#define ZXE_NOT_FOUND -2   // файл не найден
#define ZXE_INVALID   -3   // неверный заголовок
#define ZXE_TOO_LARGE -4   // программа > 64 KB
```

---

## 4. SDK - Создание программ

### 4.1 Структура проекта

```
developmentfolder/
├── zxe_api.h      <- единственный заголовок SDK
├── start.asm      <- точка входа (не трогать)
├── zxe.ld         <- линкер-скрипт (не трогать)
├── mkzxe          <- утилита упаковки
├── makefile       <- сборка всех .c -> .zxe
└── myprogram.c    <- ваша программа
```

### 4.2 Минимальная программа

```c
#include "zxe_api.h"

int main(int argc, char* argv[]) {
    clear();
    color(LIGHT_BLUE, BLACK);
    println("Hello, Zinux!");
    color(WHITE, BLACK);
    waitkey();
    return 0;
}
```

### 4.3 Сборка и установка

```bash
# Сборка всех .c файлов в .zxe
make

# Установка на образ диска
make install

# Запуск Zinux в QEMU (из корня проекта)
cd .. && make run

# Подготовка mkzxe (один раз, если отсутствует)
gcc -o mkzxe mkzxe.c
```

### 4.4 Флаги компилятора

```makefile
CFLAGS = -m32 -std=c11 -ffreestanding -fno-stack-protector -fno-pie -Wall -Wextra -O0 -fno-builtin -march=i486 -fno-jump-tables -c
```

### 4.5 Ограничения

- Максимальный размер программы: **64 KB**
- Нет `malloc`/`free` - только статические массивы
- Нет стандартной libc - только функции из `zxe_api.h`
- Нет прямого доступа к оборудованию - только через `int 0x80`
- Имена файлов FAT16: формат **8.3**, верхний регистр
- Только 1 уровень вложенности поддиректорий
- **`argc`/`argv` не передаются** - `zxe_run()` вызывает `run_user_program(..., 0, NULL)`. Не используйте `argv` - краш.

### 4.6 Завершение программы

`start.asm` после возврата из `main` выполняет:

```nasm
mov ebx, eax   ; exit code
mov eax, 2     ; syscall exit
int 0x80
```

Программа не должна делать `return` напрямую из `_start` - это вызовет падение. Просто делайте `return 0` из `main`, остальное `start.asm` берёт на себя.

---

## 5. API Reference

### 5.1 Цвета

Цвета - **32-bit RGB** (`uint32_t 0x00RRGGBB`).

```c
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

// Произвольный цвет:
#define RGB(r,g,b) (((uint32_t)(r)<<16)|((uint32_t)(g)<<8)|(b))

color(LIGHT_GREEN, BLACK);    // зелёный текст, чёрный фон
color(RGB(255,128,0), BLACK); // оранжевый текст
```


### 5.2 Вывод текста

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `print(s)` | `void print(const char* s)` | Вывести строку без переноса |
| `println(s)` | `void println(const char* s)` | Вывести строку + `\n` |
| `putchar(c)` | `void putchar(char c)` | Вывести один символ |
| `print_int(v)` | `void print_int(int v)` | Вывести целое число |
| `print_hex(v)` | `void print_hex(uint32_t v)` | Вывести число в hex (с `0x`) |
| `color(fg,bg)` | `void color(uint32_t fg, uint32_t bg)` | Установить цвет (RGB) |

### 5.3 Ввод с клавиатуры

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `waitkey()` | `char waitkey(void)` | Ждать нажатия |
| `input(buf,max)` | `void input(char* buf, int max)` | Ввод строки до нажатия Enter |
| `kbhit()` | `int kbhit(void)` | 1 если есть символ в буфере |
| `getch()` | `char getch(void)` | Взять символ без ожидания |
| `flushkeys()` | `void flushkeys(void)` | Очистить буфер клавиатуры |

#### Специальные клавиши

```c
#define KEY_UP        ((char)128)
#define KEY_DOWN      ((char)129)
#define KEY_LEFT      ((char)130)
#define KEY_RIGHT     ((char)131)
#define KEY_ESC        27
#define KEY_ENTER     '\n'
#define KEY_BACKSPACE '\b'
#define KEY_TAB       '\t'
#define KEY_F1        ((char)132)
#define KEY_F2        ((char)133)
#define KEY_F3        ((char)134)
#define KEY_F4        ((char)135)
#define KEY_SPACE     ' '
```

> Значения стрелок (128–131) выходят за диапазон `signed char`. Используйте `unsigned char`:
> ```c
> unsigned char uk = (unsigned char)waitkey();
> if (uk == 128) { /* KEY_UP */ }
> ```

### 5.4 Экран и курсор

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `clear()` | `void clear(void)` | Очистить экран |
| `gotoxy(x,y)` | `void gotoxy(int x, int y)` | Переместить курсор |
| `getxy(&x,&y)` | `void getxy(int* x, int* y)` | Получить позицию курсора |
| `screen_w()` | `int screen_w(void)` | Ширина экрана в символах / пикселях |
| `screen_h()` | `int screen_h(void)` | Высота экрана в строках / пикселях |

```c
#define SCREEN_COLS  80
#define SCREEN_ROWS  30
// (0,0) - левый верхний угол
```

### 5.5 TUI - Примитивы рисования

| Функция | Описание |
|---------|----------|
| `box(x,y,w,h,fg,bg)` | Рамка без заголовка |
| `box_titled(x,y,w,h,title,fg,bg)` | Рамка с заголовком |
| `fill(x,y,w,h,c,fg,bg)` | Заполнить прямоугольник символом |
| `hline(x,y,len,fg,bg)` | Горизонтальная линия |
| `vline(x,y,len,fg,bg)` | Вертикальная линия |

#### Символы псевдографики

```c
#define CHAR_HLINE  '\xC4'  // ─
#define CHAR_VLINE  '\xB3'  // │
#define CHAR_TL     '\xDA'  // ┌
#define CHAR_TR     '\xBF'  // ┐
#define CHAR_BL     '\xC0'  // └
#define CHAR_BR     '\xD9'  // ┘
#define CHAR_SHADE  '\xB0'  // ░
#define CHAR_BLOCK  '\xDB'  // █
```

### 5.6 TUI - Готовые компоненты

| Функция | Описание |
|---------|----------|
| `window(x,y,w,h,title)` | Окно с заголовком |
| `button(x,y,text,sel)` | Кнопка (sel=1 - выделена) |
| `textbox(x,y,w,text,active)` | Поле ввода |
| `progress(x,y,w,pct)` | Прогресс-бар 0–100% |
| `msgbox(title,msg)` | Диалог с кнопкой OK |
| `center_x(text)` | X для центрирования строки |
| `label(x,y,text,color)` | Текстовый лейбл |
| `checkbox(x,y,text,val)` | Чекбокс |
| `menubar(y,items,n,sel)` | Горизонтальное меню |
| `tabbar(x,y,w,tabs,n,active)` | Вкладки |
| `listbox(x,y,w,h,items,n,sel)` | Список |
| `scrollbar(x,y,h,total,visible,pos)` | Полоса прокрутки |
| `popup_menu(x,y,items,n,sel)` | Выпадающее меню |
| `titlebar(x,y,w,title,active)` | Заголовок окна |
| `draw_cursor(x,y)` | Нарисовать курсор мыши |

### 5.7 Строки и память

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `streq(a,b)` | `int streq(const char*, const char*)` | 1 если строки равны |
| `slen(s)` | `int slen(const char*)` | Длина строки |
| `scopy(dst,src)` | `void scopy(char*, const char*)` | Копировать строку |
| `str_cat(dst,src)` | `void str_cat(char*, const char*)` | Конкатенация строк |
| `str_ncmp(a,b,n)` | `int str_ncmp(const char*, const char*, int)` | Сравнение n символов |
| `to_int(s)` | `int to_int(const char*)` | Строка -> int |
| `to_str(v,buf)` | `void to_str(int, char*)` | int -> строка |

```c
// mem_copy и mem_set - только через прямой syscall:
__api3(SYSCALL_API_MEM_COPY, (uint32_t)dst, (uint32_t)src, n);
__api3(SYSCALL_API_MEM_SET,  (uint32_t)dst, val, n);
```

### 5.8 Файловая система (FAT16)

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `fexists(fn)` | `int fexists(const char*)` | 1 если файл существует |
| `fsize(fn)` | `uint32_t fsize(const char*)` | Размер файла в байтах |
| `fread(fn,buf,max)` | `int fread(const char*, void*, uint32_t)` | Читать файл -> байт или -1 |
| `fwrite(fn,buf,n)` | `int fwrite(const char*, const void*, uint32_t)` | Записать файл -> 0 или -1 |

```c
// Имена ТОЛЬКО в верхнем регистре, формат 8.3:
fread("DATA.TXT", buf, 512);
fread("CONF/USER.CFG", buf, 512);
```

### 5.9 Время и задержки

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `get_time(&h,&m,&s)` | `void get_time(uint8_t*, uint8_t*, uint8_t*)` | Время из RTC |
| `get_date(&d,&m,&y)` | `void get_date(uint8_t*, uint8_t*, uint16_t*)` | Дата из RTC |
| `delay(ms)` | `void delay(uint32_t ms)` | Задержка в миллисекундах |

```c
uint8_t h, m, s;
get_time(&h, &m, &s);
if (h < 10) putchar('0'); print_int(h);
putchar(':');
if (m < 10) putchar('0'); print_int(m);
```

---

## 6. Системная информация

```c
typedef struct {
    char     username[32];   // имя пользователя
    char     hostname[32];   // имя хоста
    char     os_name[32];    // "Zinux"
    char     os_version[16]; // "0.2a"
    char     cwd[64];        // текущая директория ("/", "/BIN")
    uint32_t mem_kb;         // RAM в килобайтах
} sys_info_t;
```
> Наименование версий: a - alpha (недоступная в репозиториях), b - beta (доступная в beta branch)

| Функция | Описание |
|---------|----------|
| `sysinfo(&info)` | Заполнить всю структуру одним вызовом |
| `get_username(buf)` | Имя пользователя -> `buf[32]` |
| `get_hostname(buf)` | Имя хоста -> `buf[32]` |
| `get_cwd(buf)` | Текущая директория -> `buf[64]` |
| `get_os_name(buf)` | Название ОС -> `buf[32]` |
| `get_mem_kb()` | Объём RAM в KB |
| `set_username(name)` | Сменить пользователя (-> диск) |
| `set_hostname(name)` | Сменить хост (-> диск) |

```c
sys_info_t info;
sysinfo(&info);
print(info.username); print("@"); println(info.hostname);
print("RAM: "); print_int(info.mem_kb); println(" KB");
```

**Динамическое разрешение экрана** - через `screen_w()` и `screen_h()`:

```c
// НЕ хардкодьте "800x600"
int sw = screen_w();
int sh = screen_h();
// Используйте sw/sh для центрирования окон, расчёта позиций
```

---

## 7. Пиксельная графика или GUI

| Функция | Сигнатура | Описание |
|---------|-----------|----------|
| `pixel(x,y,c)` | `void pixel(int,int,uint32_t)` | Нарисовать пиксель |
| `line(x0,y0,x1,y1,c)` | `void line(int,int,int,int,uint32_t)` | Линия (алгоритм Брезенхема) |
| `rect(x,y,w,h,c)` | `void rect(int,int,int,int,uint32_t)` | Контур прямоугольника |
| `fillrect(x,y,w,h,c)` | `void fillrect(int,int,int,int,uint32_t)` | Залитый прямоугольник |
| `circle(cx,cy,r,c)` | `void circle(int,int,int,uint32_t)` | Контур окружности |
| `fillcircle(cx,cy,r,c)` | `void fillcircle(int,int,int,uint32_t)` | Залитая окружность |
| `gfx_char(x,y,c,fg,bg)` | `void gfx_char(int,int,char,uint32_t,uint32_t)` | Символ в пикселях (8×8) |
| `gfx_text(x,y,s,fg,bg)` | `void gfx_text(int,int,const char*,uint32_t,uint32_t)` | Строка в пикселях |
| `blit(x,y,w,h,pixels)` | `void blit(int,int,int,int,const uint32_t*)` | Блок пикселей (спрайт) |

```c
int main(int argc, char* argv[]) {
    int w = screen_w(), h = screen_h();
    fillrect(0, 0, w, h, BLACK);
    fillcircle(w/2, h/2, 60, LIGHT_BLUE);
    circle(w/2, h/2, 60, WHITE);
    gfx_text(10, 10, "Hello GFX!", WHITE, BLACK);
    waitkey();
    return 0;
}
```

**Пример GUI-окна** (паттерн из zalc):

```c
// Центрируем окно динамически
int win_x = (screen_w() - WIN_W) / 2;
int win_y = (screen_h() - WIN_H) / 2;

fillrect(win_x, win_y, WIN_W, WIN_H, RGB(45,45,60));
rect(win_x, win_y, WIN_W, WIN_H, RGB(80,80,120));

// Заголовок
fillrect(win_x, win_y, WIN_W, 28, RGB(80,80,120));
gfx_text(win_x + 8, win_y + 10, "My Window", RGB(160,200,255), RGB(80,80,120));
```

---

## 8. Мышь

```c
typedef struct {
    int     x;
    int     y;
    uint8_t buttons;
    int     wheel;
} __attribute__((packed)) mouse_state_t;

#define MOUSE_BTN_LEFT   (1 << 0)
#define MOUSE_BTN_RIGHT  (1 << 1)
#define MOUSE_BTN_MIDDLE (1 << 2)
```

| Функция | Описание |
|---------|----------|
| `mouse_get(&ms)` | Получить состояние мыши |
| `mouse_move(x,y)` | Переместить курсор |
| `mouse_sens(sx,sy)` | Установить чувствительность |
| `mouse_left()` | 1 если нажата ЛКМ |
| `mouse_right()` | 1 если нажата ПКМ |
| `draw_cursor(x,y)` | Нарисовать курсор |

---

## 9. ID системных вызовов

Для прямого вызова через `__apiN()`:

```c
SYSCALL_API_PRINT             SYSCALL_API_PRINTLN
SYSCALL_API_PRINT_CHAR        SYSCALL_API_PRINT_INT
SYSCALL_API_PRINT_HEX         SYSCALL_API_SET_COLOR
SYSCALL_API_READ_CHAR         SYSCALL_API_READ_LINE
SYSCALL_API_CLEAR_SCREEN      SYSCALL_API_SET_CURSOR
SYSCALL_API_GET_CURSOR        SYSCALL_API_DRAW_BOX
SYSCALL_API_DRAW_BOX_TITLED   SYSCALL_API_FILL_RECT
SYSCALL_API_DRAW_HLINE        SYSCALL_API_DRAW_VLINE
SYSCALL_API_DRAW_WINDOW       SYSCALL_API_DRAW_BUTTON
SYSCALL_API_DRAW_INPUT        SYSCALL_API_DRAW_PROGRESS
SYSCALL_API_STR_COMPARE       SYSCALL_API_STR_LENGTH
SYSCALL_API_STR_COPY          SYSCALL_API_MEM_COPY
SYSCALL_API_MEM_SET           SYSCALL_API_GET_TIME
SYSCALL_API_GET_DATE          SYSCALL_API_DELAY
SYSCALL_API_FILE_EXISTS       SYSCALL_API_FILE_READ
SYSCALL_API_FILE_SIZE         SYSCALL_API_FILE_WRITE
SYSCALL_API_HAS_KEY           SYSCALL_API_GET_KEY
SYSCALL_API_CLEAR_KEYS        SYSCALL_API_GET_SYSINFO
SYSCALL_API_SET_USERNAME      SYSCALL_API_SET_HOSTNAME
SYSCALL_API_GFX_PIXEL         SYSCALL_API_GFX_LINE
SYSCALL_API_GFX_RECT          SYSCALL_API_GFX_FILLRECT
SYSCALL_API_GFX_CIRCLE        SYSCALL_API_GFX_FILLCIRCLE
SYSCALL_API_GFX_CHAR          SYSCALL_API_GFX_TEXT
SYSCALL_API_GFX_WIDTH         SYSCALL_API_GFX_HEIGHT
SYSCALL_API_GFX_BLIT          SYSCALL_API_GUI_LABEL
SYSCALL_API_GUI_CHECKBOX      SYSCALL_API_GUI_MENUBAR
SYSCALL_API_GUI_TABBAR        SYSCALL_API_GUI_DRAW_CURSOR
SYSCALL_API_GUI_LISTBOX       SYSCALL_API_GUI_SCROLLBAR
SYSCALL_API_GUI_POPUP_MENU    SYSCALL_API_GUI_TITLEBAR
SYSCALL_API_GUI_DESKTOP_BAR   SYSCALL_API_GUI_ICON
SYSCALL_API_STR_CONCAT        SYSCALL_API_STR_NCOMPARE
SYSCALL_API_STR_TO_INT        SYSCALL_API_INT_TO_STR
SYSCALL_API_MOUSE_GET_STATE   SYSCALL_API_MOUSE_SET_POS
SYSCALL_API_MOUSE_SET_SENS
```

---

## 10. Команды оболочки

| Команда | Описание |
|---------|----------|
| `help` | Список всех команд |
| `ls [dir]` | Содержимое директории |
| `cat <file>` | Вывести файл на экран |
| `mkdir <dir>` | Создать директорию |
| `cd <dir>` | Сменить текущую директорию |
| `echo <text>` | Вывести текст |
| `clear` | Очистить экран |
| `info` | Информация о системе |
| `mem` | Оперативная память |
| `cpu` | Информация о процессоре |
| `time` | Текущее время |
| `date` | Текущая дата |
| `version` | Версия Zinux |
| `set user=<n>` | Установить имя пользователя |
| `set host=<n>` | Установить имя хоста |
| `reboot` | Перезагрузка |
| `halt` | Выключение |
| `sync` | Сохранить файлы из RAM на диск |
| `<program>` | Запустить `.ZXE` программу |

---

## 11. Kernel Panic

### 11.1 Обрабатываемые исключения

| INT | Название | Описание |
|-----|----------|----------|
| `#0` | Division By Zero | Деление на ноль |
| `#1` | Debug | Отладочное исключение |
| `#2` | NMI | Немаскируемое прерывание |
| `#5` | Bound Range Exceeded | Инструкция BOUND - нарушение границ массива |
| `#6` | Invalid Opcode | Недопустимая инструкция |
| `#7` | Device Not Available | FPU недоступен |
| `#8` | Double Fault | Ошибка при обработке исключения |
| `#12` | Stack-Segment Fault | Ошибка стека |
| `#13` | General Protection | Доступ к защищённой памяти / нарушение Ring |
| `#14` | Page Fault | Ошибка страницы |

### 11.2 USER FAULT и KERNEL PANIC

- **`[USER FAULT]`** - исключение в Ring 2 (программа). Ядро перехватывает, показывает тип ошибки и возвращается в оболочку. Система продолжает работу.
- **`!!!KERNEL PANIC!!!`** - исключение в Ring 0 (ядро). Система останавливается.

```
[USER FAULT] General Protection Fault   <- программа нарушила Ring
[USER FAULT] Bound Range Exceeded       <- скомпилировано с -O2 вместо -O0
[USER FAULT] Invalid Opcode             <- неверные инструкции / старый API
```

### 11.3 Диагностика по EIP

```
0x10000–0x1FFFF   -> код ядра
0x200030–0x20FFFF -> код .ZXE программы
0x00000000        -> разыменование нулевого указателя
```

---

## 12. Кириллица и русский язык

Ядро использует кодировку **CP866**.

```
Заглавные (0x80–0x9F):
  А Б В Г Д Е Ё Ж З И Й К Л М Н О П Р С Т У Ф Х Ц Ч Ш Щ Ъ Ы Ь Э Ю Я

Строчные:
  а б в г д е ё ж з и й к л м н о п р с т у ф х ц ч ш щ ъ ы ь э ю я
```

> С Zinux 0.2 escape последовательность больше не нужна.
> Пример:
```C 
print("А Б В Г");
```
---

## 13. Известные ограничения и решения

| Проблема | Причина | Решение |
|----------|---------|---------|
| `[USER FAULT] Bound Range Exceeded` после выхода | Флаг `-O2` генерирует инструкцию `BOUND` | Используйте `-O0` в makefile |
| `[USER FAULT] Page Fault at 0x6F6C6C65` | Программа прыгает в строку после `main` | Обновите `start.asm` - нужен `int 0x80` exit вместо `ret` |
| `[USER FAULT] General Protection Fault` | IDT gate `int 0x80` имеет `DPL=0` вместо `DPL=3` | `idt_set_gate(0x80, ..., 0xEE)` в `kernel/system.c` |
| Стрелки не работают | `signed char` обрезает значения 128–131 | `unsigned char uk = (unsigned char)waitkey()` |