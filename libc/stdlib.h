#ifndef LIBC_STDLIB_H
#define LIBC_STDLIB_H

#include "types.h"

// Конвертация строк в числа
int          atoi(const char* s);
long         atol(const char* s);
long long    atoll(const char* s);
long         strtol(const char* s, char** endptr, int base);
unsigned long strtoul(const char* s, char** endptr, int base);

// Числа в строку (не стандарт, но удобно)
void itoa(int value, char* buf, int base);
void utoa(unsigned int value, char* buf, int base);
void ltoa(long value, char* buf, int base);

// Арифметика
int  abs(int x);
long labs(long x);

// Псевдослучайные числа (LCG)
#define RAND_MAX 0x7FFFFFFF
int  rand(void);
void srand(unsigned int seed);

// Сортировка и поиск
void  qsort(void* base, size_t n, size_t size,
            int (*cmp)(const void*, const void*));
void* bsearch(const void* key, const void* base, size_t n, size_t size,
              int (*cmp)(const void*, const void*));

// Завершение (в ядре вызывает halt)
void exit(int code) __attribute__((noreturn));
void abort(void)    __attribute__((noreturn));

#endif