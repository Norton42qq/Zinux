#include "memory.h"
#include "string.h"

// Заголовок блока кучи
typedef struct block {
    size_t        size;   // размер полезной нагрузки (без заголовка)
    uint8_t       free;
    struct block* next;
} block_t;

#define BLOCK_HDR sizeof(block_t)
#define MIN_SPLIT 16  // минимальный размер для разбиения блока

// Символ из linker.ld — начало кучи
// Объявляем как массив нулевого размера чтобы не вводить GCC в заблуждение
extern char _heap_start[];

static block_t* heap_head = NULL;

void memory_init(void) {
    heap_head = (block_t*)_heap_start;
    heap_head->size = HEAP_END - (uint32_t)_heap_start - BLOCK_HDR;
    heap_head->free = 1;
    heap_head->next = NULL;
}

// Слияние соседних свободных блоков
static void coalesce(void) {
    block_t* cur = heap_head;
    while (cur && cur->next) {
        if (cur->free && cur->next->free) {
            cur->size += BLOCK_HDR + cur->next->size;
            cur->next  = cur->next->next;
        } else {
            cur = cur->next;
        }
    }
}

void* malloc(size_t size) {
    if (size == 0) return NULL;

    // Выравниваем до 4 байт
    size = (size + 3) & ~3u;

    block_t* cur = heap_head;
    while (cur) {
        if (cur->free && cur->size >= size) {
            // Разбиваем блок если остаток достаточно большой
            if (cur->size >= size + BLOCK_HDR + MIN_SPLIT) {
                block_t* split = (block_t*)((uint8_t*)cur + BLOCK_HDR + size);
                split->size = cur->size - size - BLOCK_HDR;
                split->free = 1;
                split->next = cur->next;

                cur->size = size;
                cur->next = split;
            }
            cur->free = 0;
            return (uint8_t*)cur + BLOCK_HDR;
        }
        cur = cur->next;
    }

    return NULL; // OOM
}

void free(void* ptr) {
    if (!ptr) return;
    block_t* blk = (block_t*)((uint8_t*)ptr - BLOCK_HDR);
    blk->free = 1;
    coalesce();
}

void* calloc(size_t count, size_t size) {
    size_t total = count * size;
    void* ptr = malloc(total);
    if (ptr) memset(ptr, 0, total);
    return ptr;
}

void* realloc(void* ptr, size_t size) {
    if (!ptr)   return malloc(size);
    if (!size)  { free(ptr); return NULL; }

    block_t* blk = (block_t*)((uint8_t*)ptr - BLOCK_HDR);
    if (blk->size >= size) return ptr; // текущий блок достаточно велик

    void* new_ptr = malloc(size);
    if (!new_ptr) return NULL;

    memcpy(new_ptr, ptr, blk->size);
    free(ptr);
    return new_ptr;
}

size_t heap_used(void) {
    size_t used = 0;
    block_t* cur = heap_head;
    while (cur) {
        if (!cur->free) used += cur->size;
        cur = cur->next;
    }
    return used;
}

size_t heap_free(void) {
    size_t free_sz = 0;
    block_t* cur = heap_head;
    while (cur) {
        if (cur->free) free_sz += cur->size;
        cur = cur->next;
    }
    return free_sz;
}

// Диагностический вывод карты блоков
void heap_dump(void) {
    // подключается к vesa_print через stdio/printf
    extern int printf(const char* fmt, ...);
    block_t* cur = heap_head;
    int i = 0;
    printf("heap dump:\n");
    while (cur) {
        printf("  [%d] addr=0x%x size=%d %s\n",
            i++,
            (uint32_t)((uint8_t*)cur + BLOCK_HDR),
            cur->size,
            cur->free ? "free" : "used");
        cur = cur->next;
    }
}