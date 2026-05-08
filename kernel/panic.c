#include "panic.h"
#include "drivers/video.h"
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

#define PANIC_COLOR_BG      RGB( 15,  24,  48)   // Тёмно-синий
#define PANIC_COLOR_TEXT    RGB(180, 200, 255)   // Светло-голубой
#define PANIC_COLOR_WHITE   RGB(255, 255, 255)   // Белый
#define PANIC_COLOR_REG_KEY RGB(255, 220,  50)   // Жёлтый

static void put_hex(uint32_t v) {
    const char h[] = "0123456789ABCDEF";
    video_print("0x");
    for (int i = 28; i >= 0; i -= 4)
        video_print_char(h[(v >> i) & 0xF]);
}

static void panic_screen(const char* title, const char* msg, registers_t* regs) {
    disable_interrupts();

    video_clear(PANIC_COLOR_BG);

    video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
    for (int i = 0; i < 100; i++) video_print_char(' ');
    video_set_cursor(0, 1);
    for (int i = 0; i < 100; i++) video_print_char(' ');
    video_set_cursor(2, 1);
    video_print("!!!KERNEL PANIC!!!");

    video_set_color(PANIC_COLOR_TEXT, PANIC_COLOR_BG);

    video_set_cursor(0, 3);
    video_print("  Exception: ");
    video_print(title);

    if (msg) {
        video_set_cursor(0, 4);
        video_print("  "); video_print(msg);
    }

    if (regs) {
        video_set_color(PANIC_COLOR_TEXT, PANIC_COLOR_BG);
        video_set_cursor(0, 6);
        video_print("  Error code: "); put_hex(regs->err_code);
        video_print("    EIP: "); put_hex(regs->eip);

        video_set_cursor(2, 8);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("EAX=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->eax);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("  EBX=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ebx);

        video_set_cursor(2, 9);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("ECX=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ecx);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("  EDX=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->edx);

        video_set_cursor(2, 10);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("ESI=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->esi);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("  EDI=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->edi);

        video_set_cursor(2, 11);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("EBP=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ebp);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("  ESP=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->esp);

        video_set_cursor(2, 12);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("EIP=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->eip);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("  EFL=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->eflags);

        video_set_cursor(2, 13);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(" CS=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->cs);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("   SS=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ss);

        video_set_cursor(2, 14);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print(" DS=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->ds);
        video_set_color(PANIC_COLOR_REG_KEY, PANIC_COLOR_BG);
        video_print("   ES=");
        video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
        put_hex(regs->es);
    }

    video_set_color(PANIC_COLOR_WHITE, PANIC_COLOR_BG);
    video_set_cursor(0, 17);
    video_print("The system has stopped working. Restart your computer.\n\n");

    video_print("            %%%%%                \n          %*     %               \n          %    -...%%\n          %#  %..%:...*%*\n           %   %......%+..%      \n           %%   .%.......%%      \n         %%  %-   %%....%        \n       #%     %     %            \n      %*      %      %           \n      % %    %       %           \n     % % %%%  +%%    %#%*        \n     % %%    .....% %:..%        \n        %%:  %....*%...%         \n           *%%%%%#  %%           ");

    system_halt();
}

void exception_handler(registers_t* regs) {
    const char* name = "Unknown Exception";
    if (regs->int_num < 20)
        name = exception_names[regs->int_num];

    // Если это исключение пришло из Ring2 - не вызывать панику
    // Сделать возврат программы будто она завершилась с ошибкой
    if ((regs->cs & 0x3) == 2) {
        video_set_color(COLOR_LIGHT_RED, COLOR_BLACK);
        video_print("\n[PROGRAM FAULT] ");
        video_set_color(COLOR_WHITE, COLOR_BLACK);
        video_print(name);

        if (regs->int_num == 14) {
            uint32_t cr2;
            __asm__ volatile ("mov %%cr2, %0" : "=r"(cr2));
            video_print(" at ");
            put_hex(cr2);
            video_print(" err=");
            put_hex(regs->err_code);
        }
        video_print("\n");

        // 0xE0000000 | int_num - чтобы видеть причину в ZXE-загрузчике
        extern void abort_user_program(uint32_t exit_code);
        abort_user_program(0xE0000000u | (regs->int_num & 0xFFu));
        return;
    }

    // Kernel fault
    if (regs->int_num == 14) {
        uint32_t cr2;
        __asm__ volatile ("mov %%cr2, %0" : "=r"(cr2));
        video_print("\n[PF] CR2=");
        put_hex(cr2);
        video_print(" ERR=");
        put_hex(regs->err_code);
        video_print("\n");
    }
    panic_screen(name, NULL, regs);
}

void kernel_panic(const char* msg) {
    panic_screen("Kernel Panic", msg, NULL);
}