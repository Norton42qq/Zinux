#include "stdlib.h"
#include "ctype.h"
#include "string.h"

int errno = 0;  // определяем здесь, объявлен в errno.h

// ─── Конвертация строк ───────────────────────────────────────────────

long strtol(const char* s, char** endptr, int base) {
    while (isspace(*s)) s++;

    int neg = 0;
    if (*s == '-')      { neg = 1; s++; }
    else if (*s == '+') { s++; }

    // Автоопределение основания
    if (base == 0) {
        if (s[0] == '0' && (s[1] == 'x' || s[1] == 'X')) { base = 16; s += 2; }
        else if (s[0] == '0')                              { base = 8;  s++; }
        else                                               { base = 10; }
    } else if (base == 16 && s[0] == '0' && (s[1] == 'x' || s[1] == 'X')) {
        s += 2;
    }

    long result = 0;
    const char* start = s;

    while (*s) {
        int digit;
        if      (isdigit(*s))             digit = *s - '0';
        else if (*s >= 'a' && *s <= 'f')  digit = *s - 'a' + 10;
        else if (*s >= 'A' && *s <= 'F')  digit = *s - 'A' + 10;
        else break;

        if (digit >= base) break;
        result = result * base + digit;
        s++;
    }

    if (endptr) *endptr = (s == start) ? (char*)start : (char*)s;
    return neg ? -result : result;
}

unsigned long strtoul(const char* s, char** endptr, int base) {
    while (isspace(*s)) s++;
    if (*s == '+') s++;

    if (base == 0) {
        if (s[0] == '0' && (s[1] == 'x' || s[1] == 'X')) { base = 16; s += 2; }
        else if (s[0] == '0')                              { base = 8;  s++; }
        else                                               { base = 10; }
    } else if (base == 16 && s[0] == '0' && (s[1] == 'x' || s[1] == 'X')) {
        s += 2;
    }

    unsigned long result = 0;
    const char* start = s;

    while (*s) {
        int digit;
        if      (isdigit(*s))            digit = *s - '0';
        else if (*s >= 'a' && *s <= 'f') digit = *s - 'a' + 10;
        else if (*s >= 'A' && *s <= 'F') digit = *s - 'A' + 10;
        else break;

        if (digit >= base) break;
        result = result * base + digit;
        s++;
    }

    if (endptr) *endptr = (s == start) ? (char*)start : (char*)s;
    return result;
}

int   atoi(const char* s)  { return (int)strtol(s, NULL, 10); }
long  atol(const char* s)  { return strtol(s, NULL, 10); }
long long atoll(const char* s) { return (long long)strtol(s, NULL, 10); }

// ─── Числа в строку ──────────────────────────────────────────────────

void itoa(int val, char* buf, int base) {
    char* p = buf;
    if (val < 0 && base == 10) { *p++ = '-'; val = -val; }
    char* start = p;
    if (val == 0) { *p++ = '0'; }
    else {
        while (val) { *p++ = "0123456789abcdef"[val % base]; val /= base; }
    }
    *p-- = '\0';
    // переворачиваем
    while (start < p) { char t = *start; *start++ = *p; *p-- = t; }
}

void utoa(unsigned int val, char* buf, int base) {
    char* p = buf;
    char* start = p;
    if (val == 0) { *p++ = '0'; }
    else {
        while (val) { *p++ = "0123456789abcdef"[val % base]; val /= base; }
    }
    *p-- = '\0';
    while (start < p) { char t = *start; *start++ = *p; *p-- = t; }
}

void ltoa(long val, char* buf, int base) { itoa((int)val, buf, base); }

// ─── Арифметика ──────────────────────────────────────────────────────

int  abs(int x)   { return x < 0 ? -x : x; }
long labs(long x) { return x < 0 ? -x : x; }

// ─── Псевдослучайные числа (LCG) ────────────────────────────────────

static unsigned int _rand_seed = 12345;

int rand(void) {
    _rand_seed = _rand_seed * 1664525u + 1013904223u;
    return (int)(_rand_seed & RAND_MAX);
}

void srand(unsigned int seed) {
    _rand_seed = seed;
}

// ─── Сортировка ──────────────────────────────────────────────────────

// Вспомогательная функция: обмен блоков памяти
static void swap_bytes(uint8_t* a, uint8_t* b, size_t size) {
    while (size--) { uint8_t t = *a; *a++ = *b; *b++ = t; }
}

// Introsort = quicksort с fallback на insertion sort при малых массивах
static void insertion_sort(uint8_t* base, size_t n, size_t size,
                           int (*cmp)(const void*, const void*)) {
    for (size_t i = 1; i < n; i++) {
        for (size_t j = i; j > 0; j--) {
            uint8_t* a = base + (j - 1) * size;
            uint8_t* b = base + j * size;
            if (cmp(a, b) <= 0) break;
            swap_bytes(a, b, size);
        }
    }
}

static void quicksort(uint8_t* base, size_t n, size_t size,
                      int (*cmp)(const void*, const void*)) {
    if (n < 8) { insertion_sort(base, n, size, cmp); return; }

    // Опорный элемент — середина
    uint8_t* pivot = base + (n / 2) * size;
    uint8_t* left  = base;
    uint8_t* right = base + (n - 1) * size;

    while (left <= right) {
        while (cmp(left,  pivot) < 0) left  += size;
        while (cmp(right, pivot) > 0) right -= size;
        if (left <= right) {
            swap_bytes(left, right, size);
            // Обновляем pivot если он сдвинулся
            if (pivot == left)  pivot = right;
            else if (pivot == right) pivot = left;
            left  += size;
            right -= size;
        }
    }

    size_t left_n  = (right - base) / size + 1;
    size_t right_n = n - (left - base) / size;

    if (left_n  > 1) quicksort(base,  left_n,  size, cmp);
    if (right_n > 1) quicksort(left,  right_n, size, cmp);
}

void qsort(void* base, size_t n, size_t size,
           int (*cmp)(const void*, const void*)) {
    if (n < 2 || size == 0) return;
    quicksort((uint8_t*)base, n, size, cmp);
}

// ─── Бинарный поиск ──────────────────────────────────────────────────

void* bsearch(const void* key, const void* base, size_t n, size_t size,
              int (*cmp)(const void*, const void*)) {
    const uint8_t* lo = base;
    const uint8_t* hi = lo + (n - 1) * size;

    while (lo <= hi) {
        const uint8_t* mid = lo + ((hi - lo) / size / 2) * size;
        int r = cmp(key, mid);
        if      (r < 0) hi = mid - size;
        else if (r > 0) lo = mid + size;
        else            return (void*)mid;
    }
    return NULL;
}

// ─── Завершение ──────────────────────────────────────────────────────

extern void system_halt(void) __attribute__((noreturn));

void exit(int code) {
    (void)code;
    system_halt();
}

void abort(void) {
    system_halt();
}