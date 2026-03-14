#ifndef SYSTEM_H
#define SYSTEM_H

#include <stdint.h>

typedef struct {
    uint16_t offset_low;
    uint16_t selector;
    uint8_t zero;
    uint8_t type_attr;
    uint16_t offset_high;
} __attribute__((packed)) idt_entry_t;

typedef struct {
    uint16_t limit;
    uint32_t base;
} __attribute__((packed)) idt_ptr_t;

void idt_init(void);
void idt_set_gate(int num, uint32_t handler, uint16_t selector, uint8_t flags);
void pic_init(void);
void pic_send_eoi(uint8_t irq);

typedef struct {
    uint32_t mem_lower;         // Память ниже 1MB (KB)
    uint32_t mem_upper;         // Память выше 1MB (KB)
    uint32_t cpu_vendor[4];     // Строка производителя CPU
    uint32_t cpu_features;      // Флаги функций CPU
} system_info_t;

void system_init(void);
system_info_t* system_get_info(void);
void system_halt(void);
void system_reboot(void);

extern void load_idt(idt_ptr_t* idt_ptr);
extern void enable_interrupts(void);
extern void disable_interrupts(void);
extern void keyboard_handler(void);

// Обработчики исключений из kernel_entry.asm
extern void isr0(void);  extern void isr1(void);
extern void isr2(void);  extern void isr3(void);
extern void isr4(void);  extern void isr5(void);
extern void isr6(void);  extern void isr7(void);
extern void isr8(void);  extern void isr9(void);
extern void isr10(void); extern void isr11(void);
extern void isr12(void); extern void isr13(void);
extern void isr14(void); extern void isr15(void);
extern void isr16(void); extern void isr17(void);
extern void isr18(void); extern void isr19(void);

#endif