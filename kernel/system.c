#include "system.h"
#include "io.h"
#include "string.h"

// IDT
static idt_entry_t idt[256];
static idt_ptr_t idt_ptr;

// Системная информация
static system_info_t sys_info;

// Установка записи в IDT
void idt_set_gate(int num, uint32_t handler, uint16_t selector, uint8_t flags) {
    idt[num].offset_low = handler & 0xFFFF;
    idt[num].offset_high = (handler >> 16) & 0xFFFF;
    idt[num].selector = selector;
    idt[num].zero = 0;
    idt[num].type_attr = flags;
}

// Инициализация PIC
void pic_init(void) {
    // ICW1 Инициализация
    outb(0x20, 0x11);
    io_wait();
    outb(0xA0, 0x11);
    io_wait();
    
    // ICW2 Векторные смещения
    outb(0x21, 0x20);   // Master: IRQ 0-7 - INT 32-39
    io_wait();
    outb(0xA1, 0x28);   // Slave: IRQ 8-15 - INT 40-47
    io_wait();
    
    // ICW3 Каскадирование
    outb(0x21, 0x04);
    io_wait();
    outb(0xA1, 0x02);
    io_wait();
    
    // ICW4 Режим 8086
    outb(0x21, 0x01);
    io_wait();
    outb(0xA1, 0x01);
    io_wait();
    
    // Маскировка всего IRQ кроме клавиатуры (IRQ1)
    outb(0x21, 0xFD);
    outb(0xA1, 0xFF);
}

// Отправка EOI
void pic_send_eoi(uint8_t irq) {
    if (irq >= 8) {
        outb(0xA0, 0x20);
    }
    outb(0x20, 0x20);
}

// Инициализация IDT
void idt_init(void) {
    memset(idt, 0, sizeof(idt));

    // Исключения CPU 0-19
    idt_set_gate(0,  (uint32_t)isr0,  0x08, 0x8E);
    idt_set_gate(1,  (uint32_t)isr1,  0x08, 0x8E);
    idt_set_gate(2,  (uint32_t)isr2,  0x08, 0x8E);
    idt_set_gate(3,  (uint32_t)isr3,  0x08, 0x8E);
    idt_set_gate(4,  (uint32_t)isr4,  0x08, 0x8E);
    idt_set_gate(5,  (uint32_t)isr5,  0x08, 0x8E);
    idt_set_gate(6,  (uint32_t)isr6,  0x08, 0x8E);
    idt_set_gate(7,  (uint32_t)isr7,  0x08, 0x8E);
    idt_set_gate(8,  (uint32_t)isr8,  0x08, 0x8E);
    idt_set_gate(9,  (uint32_t)isr9,  0x08, 0x8E);
    idt_set_gate(10, (uint32_t)isr10, 0x08, 0x8E);
    idt_set_gate(11, (uint32_t)isr11, 0x08, 0x8E);
    idt_set_gate(12, (uint32_t)isr12, 0x08, 0x8E);
    idt_set_gate(13, (uint32_t)isr13, 0x08, 0x8E);
    idt_set_gate(14, (uint32_t)isr14, 0x08, 0x8E);
    idt_set_gate(15, (uint32_t)isr15, 0x08, 0x8E);
    idt_set_gate(16, (uint32_t)isr16, 0x08, 0x8E);
    idt_set_gate(17, (uint32_t)isr17, 0x08, 0x8E);
    idt_set_gate(18, (uint32_t)isr18, 0x08, 0x8E);
    idt_set_gate(19, (uint32_t)isr19, 0x08, 0x8E);

    // IRQ1 = клавиатура, PIC перемаплен на 0x20 -> INT 33
    idt_set_gate(33, (uint32_t)keyboard_handler, 0x08, 0x8E);

    idt_ptr.limit = sizeof(idt) - 1;
    idt_ptr.base  = (uint32_t)&idt;
    load_idt(&idt_ptr);

    pic_init();
}

// обнаружение процессора
static void detect_cpu(void) {
    uint32_t eax, ebx, ecx, edx;
    
    // Поддержка CPUID
    __asm__ volatile (
        "pushfl\n"
        "pushfl\n"
        "xorl $0x200000, (%%esp)\n"
        "popfl\n"
        "pushfl\n"
        "popl %%eax\n"
        "xorl (%%esp), %%eax\n"
        "popfl\n"
        "andl $0x200000, %%eax\n"
        : "=a"(eax)
    );
    
    if (eax == 0) {
        // CPUID не поддерживается
        sys_info.cpu_vendor[0] = 0x6E6B6E55;
        sys_info.cpu_vendor[1] = 0x6E776F6E;
        sys_info.cpu_vendor[2] = 0;
        sys_info.cpu_features = 0;
        return;
    }
    
    // Строка производителя
    __asm__ volatile (
        "cpuid"
        : "=a"(eax), "=b"(ebx), "=c"(ecx), "=d"(edx)
        : "a"(0)
    );
    
    sys_info.cpu_vendor[0] = ebx;
    sys_info.cpu_vendor[1] = edx;
    sys_info.cpu_vendor[2] = ecx;
    sys_info.cpu_vendor[3] = 0;
    
    // Флаги функций
    __asm__ volatile (
        "cpuid"
        : "=a"(eax), "=b"(ebx), "=c"(ecx), "=d"(edx)
        : "a"(1)
    );
    
    sys_info.cpu_features = edx;
}

// Определение размера памяти
static void detect_memory(void) {
    
    sys_info.mem_lower = 640;

    outb(0x70, 0x17);
    uint8_t low = inb(0x71);
    outb(0x70, 0x18);
    uint8_t high = inb(0x71);
    
    sys_info.mem_upper = (high << 8) | low;
    
    if (sys_info.mem_upper == 0) {
        sys_info.mem_upper = 15 * 1024;
    }
}

// Инициализация системы
void system_init(void) {
    detect_cpu();
    detect_memory();
    idt_init();
}

// Получение системной информации
system_info_t* system_get_info(void) {
    return &sys_info;
}

// Остановка системы
void system_halt(void) {
    disable_interrupts();
    while (1) {
        __asm__ volatile ("hlt");
    }
}

// Перезагрузка системы
void system_reboot(void) {
    disable_interrupts();
    
    // Метод 1: Через клавиатурный контроллер
    uint8_t good = 0x02;
    while (good & 0x02) {
        good = inb(0x64);
    }
    outb(0x64, 0xFE);
    
    // Если не сработало - halt
    system_halt();
}