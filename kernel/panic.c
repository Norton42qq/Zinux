#include "panic.h"
#include "vesa.h"
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
    vesa_print("0x");
    for (int i = 28; i >= 0; i -= 4)
        vesa_print_char(h[(v >> i) & 0xF]);
}

#define PANIC_COLOR_BG        16
#define PANIC_COLOR_TEXT      17 
#define PANIC_COLOR_WHITE     19
#define PANIC_COLOR_REG_KEY   20

static void panic_setup_palette(void) {
    vesa_set_palette(PANIC_COLOR_BG,      15, 24, 48);     // Тёмно-синий
    vesa_set_palette(PANIC_COLOR_TEXT,    180, 200, 255);  // Светло-голубой
    vesa_set_palette(PANIC_COLOR_WHITE,   255, 255, 255);  // Белый
    vesa_set_palette(PANIC_COLOR_REG_KEY, 255, 220, 50);   // Жёлтый
}
static void panic_screen(const char* title, const char* msg, registers_t* regs) {
    disable_interrupts();

    panic_setup_palette();

    vesa_clear(PANIC_COLOR_BG);

    vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
    for (int i = 0; i < 100; i++) vesa_print_char(' ');
    vesa_set_cursor(0, 1);
    for (int i = 0; i < 100; i++) vesa_print_char(' ');
    vesa_set_cursor(2, 1);
    vesa_print("!!!KERNEL PANIC!!!");

    vesa_set_color(PANIC_COLOR_TEXT, PANIC_COLOR_BG);

    vesa_set_cursor(0, 3);
    vesa_print("  Exception: ");
    vesa_print(title);

    if (msg) {
        vesa_set_cursor(0, 4);
        vesa_print("  "); vesa_print(msg);
    }

    if (regs) {
        vesa_set_color(PANIC_COLOR_TEXT, PANIC_COLOR_BG);
        vesa_set_cursor(0, 6);
        vesa_print("  Error code: "); put_hex(regs->err_code);
        vesa_print("    EIP: "); put_hex(regs->eip);

        vesa_set_cursor(2, 8);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("EAX=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->eax);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("  EBX=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ebx);

        vesa_set_cursor(2, 9);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("ECX=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ecx);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("  EDX=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->edx);

        vesa_set_cursor(2, 10);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("ESI=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->esi);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("  EDI=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->edi);

        vesa_set_cursor(2, 11);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("EBP=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ebp);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("  ESP=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->esp);

        vesa_set_cursor(2, 12);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("EIP=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->eip);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("  EFL=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->eflags);

        vesa_set_cursor(2, 13);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print(" CS=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->cs);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("   SS=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ss);

        vesa_set_cursor(2, 14);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print(" DS=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ds);
        vesa_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        vesa_print("   ES=");
        vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->es);
    }

    vesa_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
    vesa_set_cursor(0, 17);
    vesa_print("The system has stopped working. Restart your computer.\n\n");

    vesa_print("            %%%%%                \n          %*     %               \n          %    -...%%\n          %#  %..%:...*%*\n           %   %......%+..%      \n           %%   .%.......%%      \n         %%  %-   %%....%        \n       #%     %     %            \n      %*      %      %           \n      % %    %       %           \n     % % %%%  +%%    %#%*        \n     % %%    .....% %:..%        \n        %%:  %....*%...%         \n           *%%%%%#  %%           ");

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