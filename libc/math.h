#ifndef LIBC_MATH_H
#define LIBC_MATH_H

#include "types.h"

// Минимум / максимум
#define MIN(a, b) ((a) < (b) ? (a) : (b))
#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define CLAMP(x, lo, hi) (MIN(MAX((x), (lo)), (hi)))

// Знак и модуль (целочисленные)
static inline int  iabs(int x)  { return x < 0 ? -x : x; }

// Целочисленное деление с округлением вверх
static inline uint32_t div_ceil(uint32_t a, uint32_t b) { return (a + b - 1) / b; }

// Выравнивание по степени двойки
static inline uint32_t align_up(uint32_t x, uint32_t align)   { return (x + align - 1) & ~(align - 1); }
static inline uint32_t align_down(uint32_t x, uint32_t align) { return x & ~(align - 1); }

// Проверка степени двойки
static inline int is_pow2(uint32_t x) { return x && !(x & (x - 1)); }

// Целочисленный квадратный корень
uint32_t isqrt(uint32_t n);

#endif