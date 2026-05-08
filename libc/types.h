#ifndef LIBC_TYPES_H
#define LIBC_TYPES_H

// Используем системный stdint.h от i686-elf-gcc — он уже правильно
// определяет uint32_t, int32_t и т.д. для нашей платформы.
// Переопределять их вручную нельзя — GCC 15 это запрещает.
#include <stdint.h>
#include <stddef.h>

// ssize_t не всегда есть в stddef, определяем сами если нужно
#ifndef _SSIZE_T_DEFINED
typedef int32_t ssize_t;
#define _SSIZE_T_DEFINED
#endif

#ifndef __cplusplus
typedef _Bool bool;
#define true  1
#define false 0
#endif

#ifndef NULL
#define NULL ((void*)0)
#endif

#define __packed     __attribute__((packed))
#define __noreturn   __attribute__((noreturn))
#define __unused     __attribute__((unused))
#define __aligned(n) __attribute__((aligned(n)))

#define sizeof_array(arr) (sizeof(arr) / sizeof((arr)[0]))

// Барьер компилятора для работы с железом
#define BARRIER() __asm__ volatile("" ::: "memory")

#endif