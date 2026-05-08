#ifndef LIBC_MEMORY_H
#define LIBC_MEMORY_H

#include "types.h"

/*
 * Аллокатор на основе свободных блоков (free list).
 * Куча начинается сразу за концом ядра (определяется через linker symbol _heap_start)
 * и растёт вверх до HEAP_END.
 *
 * Компоновщик должен экспортировать символ _heap_start —
 * добавь в linker.ld:
 *     _heap_start = .;
 * в конец секции .bss.
 *
 * HEAP_END поднят до 16MB потому что Doom требует минимум 6MB
 * только под данные. QEMU тоже нужно запускать с -m 32.
 */

#define HEAP_END 0x1000000  // 16 MB

void  memory_init(void);
void* malloc(size_t size);
void* calloc(size_t count, size_t size);
void* realloc(void* ptr, size_t size);
void  free(void* ptr);

// Диагностика
size_t heap_used(void);
size_t heap_free(void);
void   heap_dump(void);

#endif