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

typedef struct {
    uint16_t limit_low;
    uint16_t base_low;
    uint8_t  base_mid;
    uint8_t  access;
    uint8_t  gran;
    uint8_t  base_high;
} __attribute__((packed)) gdt_entry_t;

typedef struct {
    uint16_t prev_tss;
    uint16_t reserved0;
    uint32_t esp0;
    uint16_t ss0;
    uint16_t reserved1;
    uint32_t esp1;
    uint16_t ss1;
    uint16_t reserved2;
    uint32_t esp2;
    uint16_t ss2;
    uint16_t reserved3;
    uint32_t cr3;
    uint32_t eip;
    uint32_t eflags;
    uint32_t eax, ecx, edx, ebx;
    uint32_t esp, ebp, esi, edi;
    uint16_t es, reserved4;
    uint16_t cs, reserved5;
    uint16_t ss, reserved6;
    uint16_t ds, reserved7;
    uint16_t fs, reserved8;
    uint16_t gs, reserved9;
    uint16_t ldt, reserved10;
    uint16_t trap;
    uint16_t iomap_base;
} __attribute__((packed)) tss_entry_t;

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
void tss_init(void);

extern void load_idt(idt_ptr_t* idt_ptr);
extern void enable_interrupts(void);
extern void disable_interrupts(void);
extern void syscall_handler(void);
extern int  run_user_program(uint32_t entry_point, uint32_t user_stack_top, int argc, char** argv);
extern void abort_user_program(uint32_t exit_code);
extern void keyboard_handler(void);
extern void timer_handler(void);

void syscall_putc(uint32_t ch);
uint32_t syscall_api_call(uint32_t fn_id, const uint32_t* args);

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