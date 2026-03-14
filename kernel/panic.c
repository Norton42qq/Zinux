#include "panic.h"
#include "vga.h"
#include "system.h"
#include <stddef.h>

static const char* exception_names[] = {
    "Division By Zero",         "Debug",
    "Non Maskable Interrupt",   "Breakpoint",
    "Overflow",                 "Bound Range Exceeded",
    "Invalid Opcode",           "Device Not Available",
    "Double Fault",             "Coprocessor Segment",
    "Invalid TSS",              "Segment Not Present",
    "Stack-Segment Fault",      "General Protection Fault",
    "Page Fault",               "Reserved",
    "x87 FPU Error",            "Alignment Check",
    "Machine Check",            "SIMD FP Exception",
};

static void put_hex(uint32_t v) {
    const char h[] = "0123456789ABCDEF";
    vga_puts("0x");
    for (int i = 28; i >= 0; i -= 4)
        vga_putchar(h[(v >> i) & 0xF]);
}

static void panic_screen(const char* title, const char* msg, registers_t* regs) {
    disable_interrupts();

    vga_set_color(VGA_COLOR_RED, VGA_COLOR_BLACK);
    vga_clear();

    for (int i = 0; i < 80; i++) vga_putchar(' ');
    vga_set_cursor(0, 1);
    for (int i = 0; i < 80; i++) vga_putchar(' ');
    vga_set_cursor(2, 1);
    vga_puts("!!!KERNEL PANIC!!!");

    vga_set_cursor(0, 3);
    vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
    vga_puts("  Exception: ");
    vga_set_color(VGA_COLOR_YELLOW, VGA_COLOR_BLACK);
    vga_puts(title);

    if (msg) {
        vga_set_cursor(0, 4);
        vga_puts("  "); vga_puts(msg);
    }

    if (regs) {
        vga_set_color(VGA_COLOR_LIGHT_CYAN, VGA_COLOR_BLACK);
        vga_set_cursor(0, 6);
        vga_puts("  Error code: "); put_hex(regs->err_code);
        vga_puts("    EIP: "); put_hex(regs->eip);

        vga_set_color(VGA_COLOR_WHITE, VGA_COLOR_BLACK);
        vga_set_cursor(2, 8);
        vga_puts("EAX="); put_hex(regs->eax);
        vga_puts("  EBX="); put_hex(regs->ebx);
        vga_set_cursor(2, 9);
        vga_puts("ECX="); put_hex(regs->ecx);
        vga_puts("  EDX="); put_hex(regs->edx);
        vga_set_cursor(2, 10);
        vga_puts("ESI="); put_hex(regs->esi);
        vga_puts("  EDI="); put_hex(regs->edi);
        vga_set_cursor(2, 11);
        vga_puts("EBP="); put_hex(regs->ebp);
        vga_puts("  ESP="); put_hex(regs->esp);
        vga_set_cursor(2, 12);
        vga_puts("EIP="); put_hex(regs->eip);
        vga_puts("  EFL="); put_hex(regs->eflags);
        vga_set_cursor(2, 13);
        vga_puts(" CS="); put_hex(regs->cs);
        vga_puts("   SS="); put_hex(regs->ss);
        vga_set_cursor(2, 14);
        vga_puts(" DS="); put_hex(regs->ds);
        vga_puts("   ES="); put_hex(regs->es);
    }

    vga_set_color(VGA_COLOR_LIGHT_GREY, VGA_COLOR_BLACK);
    vga_set_cursor(0, 17);
    vga_puts("The system has stopped working. Restart your computer.");

    system_halt();
}

void exception_handler(registers_t* regs) {
    const char* name = "Unknown Exception";
    if (regs->int_num < 20)
        name = exception_names[regs->int_num];
    panic_screen(name, NULL, regs);
}

void kernel_panic(const char* msg) {
    panic_screen("Kernel Panic", msg, NULL);
}