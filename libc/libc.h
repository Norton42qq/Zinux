#ifndef LIBC_H
#define LIBC_H

/*
 * libc.h — Zinux Standard C Library
 *
 * Подключай этот файл вместо стандартных <stdlib.h>, <stdio.h> и т.д.
 *
 * Структура:
 *   libc/types.h   — базовые типы, NULL, size_t, bool
 *   libc/errno.h   — коды ошибок errno
 *   libc/ctype.h   — isdigit, isalpha, toupper и т.д.
 *   libc/string.h  — строковые функции
 *   libc/memory.h  — malloc / free / realloc / calloc
 *   libc/stdio.h   — printf, sprintf, snprintf, FILE, fopen...
 *   libc/stdlib.h  — atoi, atol, abs, rand, qsort, bsearch
 *   libc/math.h    — abs, min, max
 *   libc/assert.h  — assert()
 */

#include "types.h"
#include "errno.h"
#include "ctype.h"
#include "string.h"
#include "memory.h"
#include "stdio.h"
#include "stdlib.h"
#include "math.h"
#include "assert.h"

#endif