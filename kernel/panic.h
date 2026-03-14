#ifndef PANIC_H
#define PANIC_H

#include <stdint.h>

// Фрейм регистров сохранённых обработчиком исключения
typedef struct {
    uint32_t gs, fs, es, ds;
    uint32_t edi, esi, ebp, esp_dummy;
    uint32_t ebx, edx, ecx, eax;
    uint32_t int_num, err_code;
    uint32_t eip, cs, eflags, esp, ss;
} __attribute__((packed)) registers_t;

void exception_handler(registers_t* regs);

// Ручной вызов для разработчиков (или приколистов лол)
void kernel_panic(const char* msg);

#endif