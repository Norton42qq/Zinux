# Zinux - Документация по API & SDK
---

## Содержание

- [Zinux - Документация по API \& SDK](#zinux---документация-по-api--sdk)
  - [Содержание](#содержание)
  - [1. Введение](#1-введение)
  - [2. Формат ZXE](#2-формат-zxe)
    - [2.1 Структура заголовка](#21-структура-заголовка)
    - [2.2 Адрес загрузки](#22-адрес-загрузки)
    - [2.3 Коды возврата загрузчика программ](#23-коды-возврата-загрузчика-программ)
  - [3. SDK - Создание программ](#3-sdk---создание-программ)
    - [3.1 Структура проекта](#31-структура-проекта)
    - [3.2 Минимальная программа](#32-минимальная-программа)
    - [3.3 Сборка и установка](#33-сборка-и-установка)
    - [3.4 Подготовка mkzxe (один раз на случай если он отсутствует)](#34-подготовка-mkzxe-один-раз-на-случай-если-он-отсутствует)
    - [3.5 Ограничения](#35-ограничения)
    - [3.6 Адреса памяти](#36-адреса-памяти)
  - [4. API Reference](#4-api-reference)
    - [4.1 Вывод текста](#41-вывод-текста)
    - [4.2 Цвета](#42-цвета)
    - [4.3 Ввод с клавиатуры](#43-ввод-с-клавиатуры)
      - [4.3.1 Специальные клавиши](#431-специальные-клавиши)
    - [4.4 Экран и курсор](#44-экран-и-курсор)
    - [4.5 TUI - Примитивы рисования](#45-tui---примитивы-рисования)
      - [4.5.1 Символы псевдографики](#451-символы-псевдографики)
    - [4.6 TUI - Готовые компоненты](#46-tui---готовые-компоненты)
    - [4.7 Строки и память](#47-строки-и-память)
    - [4.8 Файловая система (FAT16)](#48-файловая-система-fat16)
      - [4.8.1 Соглашения](#481-соглашения)
    - [4.9 Время и задержки](#49-время-и-задержки)
  - [5. Прямой доступ к API таблице](#5-прямой-доступ-к-api-таблице)
  - [6. Полные примеры](#6-полные-примеры)
    - [Калькулятор](#калькулятор)
    - [Текстовый просмотрщик файлов](#текстовый-просмотрщик-файлов)
  - [7. Команды оболочки](#7-команды-оболочки)
    - [8. Системная информация](#8-системная-информация)
  - [9. Kernel Panic](#9-kernel-panic)
    - [9.1 Обрабатываемые исключения](#91-обрабатываемые-исключения)
    - [9.2 Пример экрана паники](#92-пример-экрана-паники)
    - [9.3 Явный вызов из кода ядра](#93-явный-вызов-из-кода-ядра)
    - [9.4 Диагностика по EIP](#94-диагностика-по-eip)
  - [10. Кириллица и русский язык](#10-кириллица-и-русский-язык)
---

## 1. Введение

Zinux - Российская 32-х битная операционная система созданная школьником по заказу Роскомнадзор и лично В.В. Путиным.
Программы компилируются в формат `.ZXE`.

Взаимодействие программы с системой происходит через **API-таблицу** - массив указателей
на функции ядра по фиксированному адресу `0x100000`. Программа никогда не обращается
к оборудованию напрямую.

```
┌─────────────────────────────────────┐
│         .ZXE программа              │
│  #include "zxe_api.h"               │
│  int main(int argc, char* argv[])   │
└──────────────┬──────────────────────┘
               │  вызов через api->print() / print() / ..
┌──────────────▼──────────────────────┐
│   API таблица  (0x100000)            │
│   массив указателей на функции ядра  │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│           Ядро Zinux                │
│   VGA · FAT16 · Keyboard · RTC ...  │
└─────────────────────────────────────┘
```

---

## 2. Формат ZXE

Файл `.ZXE` состоит из заголовка фиксированного размера **48 байт** и бинарного кода программы следом.

### 2.1 Структура заголовка

```c
typedef struct {
    uint32_t magic;        // 0x00: 0x01455A58 ("ZXE\x01")
    uint8_t  version;      // 0x04: версия формата (= 1)
    uint8_t  flags;        // 0x05: флаги (зарезервировано, = 0)
    uint16_t header_size;  // 0x06: размер заголовка (= 48)
    uint32_t code_size;    // 0x08: размер бинарного кода
    uint32_t data_size;    // 0x0C: размер секции данных (= 0)
    uint32_t bss_size;     // 0x10: размер BSS (= 0)
    uint32_t entry_offset; // 0x14: смещение точки входа (= 0)
    char     name[16];     // 0x18: имя программы (null-terminated)
    char     reserved[8];  // 0x28: зарезервировано
} __attribute__((packed)) zxe_header_t;
```

### 2.2 Адрес загрузки

Программа загружается по адресу `0x200000` (2 MB). Код начинается со смещения +48 байт.
Линкер-скрипт `zxe.ld` настраивает секцию `.text` по адресу `0x200030`.

### 2.3 Коды возврата загрузчика программ

```c
#define ZXE_OK         0   // успех
#define ZXE_ERROR     -1   // общая ошибка
#define ZXE_NOT_FOUND -2   // файл не найден
#define ZXE_INVALID   -3   // неверный заголовок
#define ZXE_TOO_LARGE -4   // программа > 64KB
```

---

## 3. SDK - Создание программ

### 3.1 Структура проекта

```
programs/
├── zxe_api.h      <- единственный заголовок SDK
├── start.asm      <- точка входа (не трогать)
├── zxe.ld         <- линкер-скрипт (не трогать)
├── mkzxe          <- утилита упаковки (собрать один раз если он отсутствует)
├── Makefile       <- сборка всех .c → .zxe
└── myprogram.c    <- ваша программа
```

### 3.2 Минимальная программа

```c
#include "zxe_api.h" # Подключение библиотеки

int main(int argc, char* argv[]) { // Вызов функции main
    clear(); // Очистка экрана
    color(LIGHT_BLUE, BLACK); // Переключение цвета по принципу color(fg, bg)
    println("Hello, Zinux!"); // Вывод текста
    color(WHITE, BLACK); // Необязательное, но желательное переключение цвета
    waitkey(); // Ожидание ввода
    return 0; // Завершение программы
}
```

### 3.3 Сборка и установка

```bash
# Сборка всех .c файлов в .zxe
make

# Установка на образ диска
make install

# Запуск Zinux в QEMU
cd .. && make run

# Запуск из шелла Zinux
user@zinux:~$ myprogram
user@zinux:~$ myprogram arg1 arg2
# Из директории bin, программа может запускаться откуда угодно
```

### 3.4 Подготовка mkzxe (один раз на случай если он отсутствует)

```bash
gcc -o mkzxe mkzxe.c
```

### 3.5 Ограничения

- Максимальный размер программы: **64 KB**
- Нет динамической памяти (`malloc`/`free`) - только статические массивы
- Нет стандартной библиотеки C - только функции из `zxe_api.h`
- Нет прямого доступа к оборудованию - только через API
- Аргументы командной строки доступны через `argc`/`argv` в `main()`
- Из-за ограничений **FAT16**, может быть только 1 уровень вложенности поддиректорий


### 3.6 Адреса памяти

```
Адрес        Размер    Содержимое
──────────────────────────────────────────────────────
0x00000      1 KB      IVT + BDA (BIOS)
0x07C00      512 B     Stage 1 (MBR загрузчик)
0x07E00      4 KB      Stage 2 (Zrub Boot Loader)
0x08000      4 B       Адрес LFB от VESA (uint32)
0x08004      4 B       Флаг RAM диска (кол-во секторов)
0x08008      2 B       VESA pitch BytesPerScanLine
0x08009      1 B       Boot flags (0x01 = safe mode)
0x10000      256 KB    Ядро Zinux
0x50000      192 KB    FAT16 RAM буфер (диск в памяти)
0x90000      ↓         Стек ядра (растёт вниз)
0xA0000                VGA/VESA область (BIOS)
0x100000     ~1 KB     API таблица (указатели функций)
0x120000     469 KB    VESA back buffer
0x200000     64 KB     Текущая ZXE программа
```
>Важно: Программа не должна обращаться к адресам ниже 0x100000 - это область ядра. Исключение: константы только для чтения из таблицы API.

---

## 4. API Reference

### 4.1 Вывод текста

Все функции выводят в текущую позицию курсора с текущим цветом.

| Обёртка | Сигнатура | Описание |
|---------|-----------|----------|
| `print(s)` | `void print(const char* s)` | Вывести строку без переноса |
| `println(s)` | `void println(const char* s)` | Вывести строку + `\n` |
| `putchar(c)` | `void putchar(char c)` | Вывести один символ |
| `print_int(v)` | `void print_int(int v)` | Вывести целое число |
| `print_hex(v)` | `void print_hex(uint32_t v)` | Вывести число в hex (с `0x`) |
| `color(fg, bg)` | `void color(uint8_t fg, uint8_t bg)` | Установить цвет текста и фона |

**Пример:**

```c
color(LIGHT_GREEN, BLACK);
print("System: ");
color(WHITE, BLACK);
println("Zinux");

print("Version: ");
print_int(1);
print("Address: ");
print_hex(0x100000);
putchar('\n');
```

---

### 4.2 Цвета

```c
#define BLACK          0    // Чёрный
#define BLUE           1    // Синий
#define GREEN          2    // Зелёный
#define CYAN           3    // Голубой
#define RED            4    // Красный
#define MAGENTA        5    // Пурпурный
#define BROWN          6    // Коричневый
#define LIGHT_GREY     7    // Светло-серый
#define DARK_GREY      8    // Тёмно-серый
#define LIGHT_BLUE     9    // Светло-синий
#define LIGHT_GREEN   10    // Светло-зелёный
#define LIGHT_CYAN    11    // Светло-голубой
#define LIGHT_RED     12    // Светло-красный
#define LIGHT_MAGENTA 13    // Светло-пурпурный
#define YELLOW        14    // Жёлтый
#define WHITE         15    // Белый
```

> **Важно:** Фон поддерживает только цвета 0–7. Значения 8–15 для `bg` вызывают мигание текста.

> **Дополнение:** Имеется возможность использования 256 цветов EGA графики 
> ```c
> set_vesa_palette(i, r, g, b); // i - index, r - red, g - green, b - blue
> ``` 

---

### 4.3 Ввод с клавиатуры

| Обёртка | Сигнатура | Описание |
|---------|-----------|----------|
| `waitkey()` | `char waitkey(void)` | Ждать нажатия и вернуть символ |
| `input(buf, max)` | `void input(char* buf, int max)` | Читать строку с эхом до Enter |
| `kbhit()` | `int kbhit(void)` | 1 если есть символ в буфере |
| `getch()` | `char getch(void)` | Взять символ без ожидания |
| `flushkeys()` | `void flushkeys(void)` | Очистить буфер клавиатуры |

#### 4.3.1 Специальные клавиши

```c
#define KEY_UP        128   // стрелка вверх
#define KEY_DOWN      129   // стрелка вниз
#define KEY_LEFT      130   // стрелка влево
#define KEY_RIGHT     131   // стрелка вправо
#define KEY_ESC        27   // Escape
#define KEY_ENTER     '\n'  // Enter
#define KEY_BACKSPACE '\b'  // Backspace
#define KEY_TAB       '\t'  // Tab
```

**Пример обработки стрелок:**

```c
int selected = 0;
while (1) {
    char k = waitkey();
    if (k == KEY_UP)    selected--;
    if (k == KEY_DOWN)  selected++;
    if (k == KEY_ENTER) break;
    if (k == KEY_ESC)   return 0;
}
```
> Если стрелки не работают, это значит значения стрелок (128–131) выходят за диапазон signed char. Используйте:
> ```c
> unsigned char uk = (unsigned char)waitkey();
> if (uk == 128) { /* вверх */ }
> ``` 
---

### 4.4 Экран и курсор

| Обёртка | Сигнатура | Описание |
|---------|-----------|----------|
| `clear()` | `void clear(void)` | Очистить экран |
| `gotoxy(x, y)` | `void gotoxy(int x, int y)` | Переместить курсор |
| `getxy(&x, &y)` | `void getxy(int* x, int* y)` | Получить позицию курсора |

```c
#define SCREEN_COLS  80   // ширина экрана в символах
#define SCREEN_ROWS  30   // высота экрана в строках
```

> Координаты `(0, 0)` - левый верхний угол. X растёт вправо, Y - вниз.

---

### 4.5 TUI - Примитивы рисования

| Обёртка | Сигнатура | Описание |
|---------|-----------|----------|
| `box(x,y,w,h,fg,bg)` | `void box(int x, int y, int w, int h, uint8_t fg, uint8_t bg)` | Рамка без заголовка |
| `box_titled(x,y,w,h,t,fg,bg)` | `void box_titled(...)` | Рамка с заголовком |
| `fill(x,y,w,h,c,fg,bg)` | `void fill(int x, int y, int w, int h, char c, uint8_t fg, uint8_t bg)` | Заполнить прямоугольник символом |
| `hline(x,y,len,fg,bg)` | `void hline(int x, int y, int len, uint8_t fg, uint8_t bg)` | Горизонтальная линия |
| `vline(x,y,len,fg,bg)` | `void vline(int x, int y, int len, uint8_t fg, uint8_t bg)` | Вертикальная линия |

#### 4.5.1 Символы псевдографики

```c
#define CHAR_HLINE  '\xC4'  // ─  горизонтальная линия
#define CHAR_VLINE  '\xB3'  // │  вертикальная линия
#define CHAR_TL     '\xDA'  // ┌  верхний левый угол
#define CHAR_TR     '\xBF'  // ┐  верхний правый угол
#define CHAR_BL     '\xC0'  // └  нижний левый угол
#define CHAR_BR     '\xD9'  // ┘  нижний правый угол
#define CHAR_SHADE  '\xB0'  // ░  штриховка
#define CHAR_BLOCK  '\xDB'  // █  полный блок
```

---

### 4.6 TUI - Готовые компоненты

| Обёртка | Сигнатура | Описание |
|---------|-----------|----------|
| `window(x,y,w,h,t)` | `void window(int x, int y, int w, int h, const char* title)` | Окно с заголовком |
| `button(x,y,t,sel)` | `void button(int x, int y, const char* text, int selected)` | Кнопка (`sel=1` - выделена) |
| `textbox(x,y,w,t,act)` | `void textbox(int x, int y, int w, const char* text, int active)` | Поле ввода |
| `progress(x,y,w,pct)` | `void progress(int x, int y, int w, int percent)` | Прогресс-бар 0–100% |
| `msgbox(title,msg)` | `void msgbox(const char* title, const char* message)` | Диалог с кнопкой OK |
| `center_x(text)` | `int center_x(const char* text)` | X для центрирования текста |

**Пример - навигационное меню:**

```c
int main(int argc, char* argv[]) {
    const char* items[] = { "Играть", "Настройки", "Выход" };
    int sel = 0, count = 3;

    while (1) {
        clear();
        window(10, 5, 40, count + 4, "Главное меню");

        for (int i = 0; i < count; i++) {
            button(14, 7 + i, items[i], sel == i);
        }

        char k = waitkey();
        if (k == KEY_UP   && sel > 0)        sel--;
        if (k == KEY_DOWN && sel < count - 1) sel++;
        if (k == KEY_ENTER) {
            if (sel == count - 1) return 0;
            msgbox("Выбор", items[sel]);
        }
    }
}
```

---

### 4.7 Строки и память

| Обёртка | Сигнатура | Описание |
|---------|-----------|----------|
| `streq(a, b)` | `int streq(const char* a, const char* b)` | 1 если строки равны |
| `slen(s)` | `int slen(const char* s)` | Длина строки |
| `scopy(dst, src)` | `void scopy(char* dst, const char* src)` | Копировать строку |
| `api->mem_copy(d,s,n)` | `void mem_copy(void* d, const void* s, uint32_t n)` | Копировать n байт |
| `api->mem_set(d,v,n)` | `void mem_set(void* d, uint8_t v, uint32_t n)` | Заполнить n байт значением |

> `mem_copy` и `mem_set` доступны только через `api->` напрямую - короткие обёртки для них не определены.

---

### 4.8 Файловая система (FAT16)


| Обёртка | Сигнатура | Описание |
|---------|-----------|----------|
| `fexists(fn)` | `int fexists(const char* fn)` | 1 если файл существует |
| `fsize(fn)` | `uint32_t fsize(const char* fn)` | Размер файла в байтах |
| `fread(fn,buf,max)` | `int fread(const char* fn, void* buf, uint32_t max)` | Читать файл, вернуть кол-во байт или -1 |
| `fwrite(fn,buf,n)` | `int fwrite(const char* fn, const void* buf, uint32_t n)` | Записать файл, вернуть 0 или -1 |

#### 4.8.1 Соглашения

- Имена файлов: формат **8.3** в верхнем регистре - `"PROG.ZXE"`, `"DATA.TXT"`
- Пути с директориями: `"CONF/USER.CFG"`
- Максимальный размер файла ограничен размером RAM буфера (~192KB)
- Запись сохраняется в RAM - на физический диск не попадёт до реализации write-back (sync)

**Пример - чтение файла:**

```c
static char buf[512];

int main(int argc, char* argv[]) {
    if (!fexists("CONF/USER.CFG")) {
        println("Файл не найден");
        waitkey();
        return 1;
    }

    int n = fread("CONF/USER.CFG", buf, 511);
    if (n < 0) {
        println("Ошибка чтения");
        waitkey();
        return 1;
    }
    buf[n] = '\0';
    println(buf);
    waitkey();
    return 0;
}
```

---

### 4.9 Время и задержки

| Обёртка | Сигнатура | Описание |
|---------|-----------|----------|
| `api->get_time(&h,&m,&s)` | `void get_time(uint8_t* h, uint8_t* m, uint8_t* s)` | Текущее время из RTC |
| `api->get_date(&d,&m,&y)` | `void get_date(uint8_t* d, uint8_t* m, uint16_t* y)` | Текущая дата из RTC |
| `delay(ms)` | `void delay(uint32_t ms)` | Задержка в миллисекундах |

**Пример:**

```c
uint8_t h, m, s;
api->get_time(&h, &m, &s);

print_int(h); putchar(':');
if (m < 10) putchar('0');
print_int(m); putchar(':');
if (s < 10) putchar('0');
print_int(s);
```

---

## 5. Прямой доступ к API таблице

`zxe_api.h` автоматически создаёт указатель `api` на адрес `0x100000`.
Все обёртки (`print`, `clear` и др.) вызывают `api->...` внутри.
Можно обращаться напрямую - результат тот же:

```c
// Эти два вызова эквивалентны:
print("Hello");
api->print("Hello");

// Напрямую нужно для функций без обёртки:
uint8_t h, m, s;
api->get_time(&h, &m, &s);

api->mem_copy(dst, src, 1024);
api->mem_set(buf, 0, sizeof(buf));
```

> `api_table_t* const api = (api_table_t*)0x100000;` - объявлен в `zxe_api.h`.
> Ядро заполняет таблицу при старте через `api_setup_table()`.

---

## 6. Полные примеры

### Калькулятор

```c
#include "zxe_api.h"

static char buf_a[32], buf_b[32];

static int parse_int(const char* s) {
    int n = 0, neg = 0;
    if (*s == '-') { neg = 1; s++; }
    while (*s >= '0' && *s <= '9') n = n * 10 + (*s++ - '0');
    return neg ? -n : n;
}

int main(int argc, char* argv[]) {
    clear();
    window(15, 6, 50, 12, "Калькулятор");

    gotoxy(17, 8);  color(LIGHT_CYAN, BLACK);
    print("A: ");   color(WHITE, BLACK);
    textbox(20, 8, 20, "", 1);
    gotoxy(20, 8);  input(buf_a, 20);

    gotoxy(17, 10); color(LIGHT_CYAN, BLACK);
    print("B: ");   color(WHITE, BLACK);
    textbox(20, 10, 20, "", 1);
    gotoxy(20, 10); input(buf_b, 20);

    int a = parse_int(buf_a);
    int b = parse_int(buf_b);

    gotoxy(17, 12); color(YELLOW, BLACK);
    print("A+B = "); print_int(a + b);
    gotoxy(17, 13);
    print("A-B = "); print_int(a - b);
    gotoxy(17, 14);
    print("A*B = "); print_int(a * b);

    gotoxy(17, 16); color(LIGHT_GREY, BLACK);
    print("Нажмите любую клавишу...");
    waitkey();
    return 0;
}
```

### Текстовый просмотрщик файлов

```c
#include "zxe_api.h"

static char content[8192];

int main(int argc, char* argv[]) {
    if (argc < 2) {
        msgbox("Ошибка", "Использование: view FILENAME");
        return 1;
    }

    int n = fread(argv[1], content, sizeof(content) - 1);
    if (n < 0) {
        msgbox("Ошибка", "Файл не найден");
        return 1;
    }
    content[n] = '\0';

    clear();
    box_titled(0, 0, SCREEN_COLS, SCREEN_ROWS - 1, argv[1],
               LIGHT_CYAN, BLACK);
    gotoxy(1, 1);
    color(WHITE, BLACK);
    print(content);

    fill(0, SCREEN_ROWS - 1, SCREEN_COLS, 1, ' ', BLACK, LIGHT_GREY);
    gotoxy(1, SCREEN_ROWS - 1);
    color(BLACK, LIGHT_GREY);
    print(" ESC - выход ");

    while (waitkey() != KEY_ESC);
    return 0;
}
```

---



## 7. Команды оболочки

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
| `sync` | Синхронизация файлов с оперативной во внутреннюю |
| `<program>` | Запустить `.ZXE` программу |

### 8. Системная информация

Получить имя пользователя, хоста, текущую директорию и объём RAM.

```c
typedef struct {
    char     username[32];   // имя пользователя
    char     hostname[32];   // имя хоста
    char     os_name[32];    // "Zinux"
    char     os_version[16]; // "2.0"
    char     cwd[64];        // текущая директория ("/" или "/BIN")
    uint32_t mem_kb;         // RAM в килобайтах
} sys_info_t;
```

| Функция | Описание |
|---------|----------|
| `sysinfo(&info)` | Заполнить структуру `sys_info_t` одним вызовом |
| `get_username(buf)` | Скопировать имя пользователя в `buf[32]` |
| `get_hostname(buf)` | Скопировать имя хоста в `buf[32]` |
| `get_cwd(buf)` | Скопировать текущую директорию в `buf[64]` |
| `get_os_name(buf)` | Скопировать название ОС в `buf[32]` |
| `get_mem_kb()` | Вернуть `uint32_t` - объём RAM в KB |
| `set_username(name)` | Сменить имя пользователя (сохраняется на диск) |
| `set_hostname(name)` | Сменить имя хоста (сохраняется на диск) |

```c
// Приветствие с именем пользователя
char user[32], host[32], cwd[64];
get_username(user);
get_hostname(host);
get_cwd(cwd);

color(LIGHT_GREEN, BLACK);
print(user); print("@"); print(host);
color(WHITE, BLACK);
print(":"); print(cwd); print("$ ");

// Полная информация одним вызовом
sys_info_t info;
sysinfo(&info);

print("ОС: ");     println(info.os_name);
print("Версия: "); println(info.os_version);
print("RAM: ");    print_int(info.mem_kb); println(" KB");

// Смена имени пользователя
set_username("norton");
// Изменение сохраняется в CONF/USER.CFG автоматически
```

---

## 9. Kernel Panic

Если в ядре или программе произошло необработанное исключение процессора,
система показывает **красный экран паники** с полной диагностикой вместо тройного сброса.

### 9.1 Обрабатываемые исключения

| INT | Название | Описание |
|-----|----------|----------|
| `#0` | Division By Zero | Деление на ноль (`idiv`, `div`) |
| `#1` | Debug | Отладочное исключение |
| `#2` | NMI | Немаскируемое прерывание |
| `#6` | Invalid Opcode | Недопустимая инструкция процессора |
| `#7` | Device Not Available | FPU недоступен |
| `#8` | Double Fault | Ошибка при обработке исключения |
| `#12` | Stack-Segment Fault | Переполнение или ошибка стека |
| `#13` | General Protection | Доступ к защищённой памяти |
| `#14` | Page Fault | Ошибка страницы (если включена пагинация) |

### 9.2 Пример экрана паники

```
 !!! KERNEL PANIC !!!

  Exception: Example panic
  Error code: 0x00000000    EIP: 0x00000000

  EAX=0x00000000  EBX=0x00000000
  ECX=0x00000000  EDX=0x00000000
  ESI=0x00000000  EDI=0x00000000
  EBP=0x00000000  ESP=0x00000000
  EIP=0x00000000  EFL=0x00000000
   CS=0x00000000   SS=0x00000000
   DS=0x00000000   ES=0x00000000

  The system has stopped working. Restart your computer..
```

### 9.3 Явный вызов из кода ядра

```c
#include "panic.h"

// В любом месте ядра
if (fat16_init(2048) != 0) {
    kernel_panic("FAT16 init failed: filesystem corrupted");
}
```

### 9.4 Диагностика по EIP

`EIP` в экране паники указывает на инструкцию, вызвавшую исключение.

- Адреса `0x10000`–`0x1FFFF` - код ядра
- Адреса `0x200030`–`0x20FFFF` - код `.ZXE` программы
- Адрес `0x00000000` - разыменование нулевого указателя

---

## 10. Кириллица и русский язык
 
Ядро использует кодировку **CP866**. Шрифт 8×16 встроен прямо в `vesa.c`
и содержит полный набор русских букв.
 
**Таблица кодов CP866 (используется в escape-последовательностях):**
 
```
Заглавные (0x80–0x9F):
  \x80=А  \x81=Б  \x82=В  \x83=Г  \x84=Д  \x85=Е  \x86=Ж  \x87=З
  \x88=И  \x89=Й  \x8A=К  \x8B=Л  \x8C=М  \x8D=Н  \x8E=О  \x8F=П
  \x90=Р  \x91=С  \x92=Т  \x93=У  \x94=Ф  \x95=Х  \x96=Ц  \x97=Ч
  \x98=Ш  \x99=Щ  \x9A=Ъ  \x9B=Ы  \x9C=Ь  \x9D=Э  \x9E=Ю  \x9F=Я
 
Строчные (0xA0–0xAF, 0xE0–0xEF):
  \xA0=а \xA1=б \xA2=в \xA3=г \xA4=д \xA5=е \xA6=ж \xA7=з
  \xA8=и \xA9=й \xAA=к \xAB=л \xAC=м \xAD=н \xAE=о \xAF=п
  \xE0=р \xE1=с \xE2=т \xE3=у \xE4=ф \xE5=х \xE6=ц \xE7=ч
  \xE8=ш \xE9=щ \xEA=ъ \xEB=ы \xEC=ь \xED=э \xEE=ю \xEF=я
 
Ё/ё: \xF0=Ё \xF1=ё
```
 
**Пример использования в программах:**
 
```c
// В .ZXE программах через API:
print("\x87\xA4\xA8\xA2\xE0\xE2\xA5\x20\x8C\xA8\xE0!");  // "Привет Мир!"
 
// В коде ядра через vesa_print():
vesa_print("\x80\x81\x82\x83\x84\x85\x86\x87");  // А Б В Г Д Е Ж З
```
 
> Если исходник сохранён в UTF-8, кириллица в строковых литералах **не будет** отображаться корректно - всегда используйте escape-коды `\xNN`.
