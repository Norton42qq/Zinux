#include "system.h"
#include "io.h"
#include "string.h"
#include "drivers/video.h"
#include "api.h"
#include "../syscall_api_ids.h"

extern void mouse_irq_handler(void);

// IDT
static idt_entry_t idt[256];
static idt_ptr_t idt_ptr;

// Системная информация
static system_info_t sys_info;
static tss_entry_t kernel_tss;

#define TSS_SELECTOR 0x28
#define USER_MEM_START 0x200000u
#define USER_MEM_END   0x300000u

#define PAGE_PRESENT 0x001u
#define PAGE_RW      0x002u
#define PAGE_USER    0x004u

static uint32_t page_directory[1024] __attribute__((aligned(4096)));
static uint32_t low_page_tables[16][1024] __attribute__((aligned(4096))); /* 64MB identity */
static uint32_t lfb_page_tables[8][1024] __attribute__((aligned(4096)));

// Установка записи в IDT
void idt_set_gate(int num, uint32_t handler, uint16_t selector, uint8_t flags) {
    idt[num].offset_low = handler & 0xFFFF;
    idt[num].offset_high = (handler >> 16) & 0xFFFF;
    idt[num].selector = selector;
    idt[num].zero = 0;
    idt[num].type_attr = flags;
}

static void gdt_set_tss_descriptor(gdt_entry_t* gdt, int index, uint32_t base, uint32_t limit) {
    gdt[index].limit_low = (uint16_t)(limit & 0xFFFF);
    gdt[index].base_low = (uint16_t)(base & 0xFFFF);
    gdt[index].base_mid = (uint8_t)((base >> 16) & 0xFF);
    gdt[index].access = 0x89; // present, ring0, available 32-bit TSS
    gdt[index].gran = (uint8_t)((limit >> 16) & 0x0F);
    gdt[index].base_high = (uint8_t)((base >> 24) & 0xFF);
}

static int user_ptr_ok(const void* p, uint32_t size) {
    if (!p) return 0;
    uint32_t a = (uint32_t)(uintptr_t)p;
    if (a < USER_MEM_START || a >= USER_MEM_END) return 0;
    if (size == 0) return 1;
    if (size > USER_MEM_END - a) return 0;
    return 1;
}

static int user_str_ok(const char* s, uint32_t max_len) {
    if (!s || max_len == 0) return 0;
    uint32_t a = (uint32_t)(uintptr_t)s;
    if (a < USER_MEM_START || a >= USER_MEM_END) return 0;
    for (uint32_t i = 0; i < max_len && (a + i) < USER_MEM_END; i++) {
        if (((const volatile char*)s)[i] == '\0') return 1;
    }
    return 0;
}

static void paging_init(void) {
    memset(page_directory, 0, sizeof(page_directory));
    memset(low_page_tables, 0, sizeof(low_page_tables));
    memset(lfb_page_tables, 0, sizeof(lfb_page_tables));

    for (uint32_t pde = 0; pde < 16; pde++) {
        uint32_t pde_flags = PAGE_PRESENT | PAGE_RW;
        uint32_t pde_start = pde << 22;
        uint32_t pde_end = pde_start + 0x400000u;
        if (pde_start < USER_MEM_END && pde_end > USER_MEM_START) pde_flags |= PAGE_USER;
        page_directory[pde] = ((uint32_t)(uintptr_t)low_page_tables[pde]) | pde_flags;

        for (uint32_t pte = 0; pte < 1024; pte++) {
            uint32_t phys = pde_start + (pte << 12);
            uint32_t flags = PAGE_PRESENT | PAGE_RW;
            if (phys >= USER_MEM_START && phys < USER_MEM_END) flags |= PAGE_USER;
            low_page_tables[pde][pte] = phys | flags;
        }
    }

    /* Keep LFB mapped after enabling paging */
    const VideoInfo* vi = video_get_info();
    if (vi && vi->lfb_addr) {
        uint32_t fb_start = vi->lfb_addr & 0xFFFFF000u;
        uint32_t fb_size = vi->pitch * vi->height;
        uint32_t fb_end = (vi->lfb_addr + fb_size + 0xFFFu) & 0xFFFFF000u;
        uint32_t start_pde = fb_start >> 22;
        uint32_t end_pde = (fb_end - 1) >> 22;
        uint32_t tidx = 0;

        for (uint32_t pde = start_pde; pde <= end_pde && tidx < 8; pde++, tidx++) {
            uint32_t base = pde << 22;
            memset(lfb_page_tables[tidx], 0, 4096);
            for (uint32_t pte = 0; pte < 1024; pte++) {
                uint32_t phys = base + (pte << 12);
                if (phys >= fb_start && phys < fb_end) {
                    lfb_page_tables[tidx][pte] = phys | PAGE_PRESENT | PAGE_RW;
                }
            }
            page_directory[pde] = ((uint32_t)(uintptr_t)lfb_page_tables[tidx]) | PAGE_PRESENT | PAGE_RW;
        }
    }

    __asm__ volatile ("mov %0, %%cr3" : : "r"((uint32_t)(uintptr_t)page_directory) : "memory");
    uint32_t cr0;
    __asm__ volatile ("mov %%cr0, %0" : "=r"(cr0));
    cr0 |= 0x80000000u;
    __asm__ volatile ("mov %0, %%cr0" : : "r"(cr0) : "memory");
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
    
    // Маскировка всего IRQ кроме таймера (IRQ0) и клавиатуры (IRQ1)
    outb(0x21, 0xFC);
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

    // IRQ0 = таймер PIT, INT 32
    idt_set_gate(32, (uint32_t)timer_handler, 0x08, 0x8E);

    // IRQ1 = клавиатура, INT 33
    idt_set_gate(33, (uint32_t)keyboard_handler, 0x08, 0x8E);
    idt_set_gate(44, (uint32_t)mouse_irq_handler, 0x08, 0x8E); // IRQ12 — PS/2 мышь
    idt_set_gate(0x80, (uint32_t)syscall_handler, 0x08, 0xEF); // ring2 -> kernel trap gate (keep IF)

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
    paging_init();
    tss_init();
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

void syscall_putc(uint32_t ch) {
    video_print_char((char)(ch & 0xFF));
}

uint32_t syscall_api_call(uint32_t fn_id, const uint32_t* a) {
    api_table_t* t = (api_table_t*)API_TABLE_ADDRESS;
    if (!t || !a) return 0;
    if (!user_ptr_ok(a, 8 * sizeof(uint32_t))) return 0;

    uint32_t arg[8];
    memcpy(arg, a, sizeof(arg));

    switch (fn_id) {
        case SYSCALL_API_PRINT:
            if (!user_str_ok((const char*)arg[0], 1024)) return 0;
            t->print((const char*)arg[0]); return 0;
        case SYSCALL_API_PRINTLN:
            if (!user_str_ok((const char*)arg[0], 1024)) return 0;
            t->println((const char*)arg[0]); return 0;
        case SYSCALL_API_PRINT_CHAR:       t->print_char((char)arg[0]); return 0;
        case SYSCALL_API_PRINT_INT:        t->print_int((int)arg[0]); return 0;
        case SYSCALL_API_PRINT_HEX:        t->print_hex(arg[0]); return 0;
        case SYSCALL_API_SET_COLOR:        t->set_color(arg[0], arg[1]); return 0;
        case SYSCALL_API_READ_CHAR:        return (uint32_t)(uint8_t)t->read_char();
        case SYSCALL_API_READ_LINE:
            if (!user_ptr_ok((void*)arg[0], arg[1] ? arg[1] : 1)) return 0;
            t->read_line((char*)arg[0], (int)arg[1]); return 0;
        case SYSCALL_API_CLEAR_SCREEN:     t->clear_screen(); return 0;
        case SYSCALL_API_SET_CURSOR:       t->set_cursor((int)arg[0], (int)arg[1]); return 0;
        case SYSCALL_API_GET_CURSOR:
            if (!user_ptr_ok((void*)arg[0], sizeof(int)) || !user_ptr_ok((void*)arg[1], sizeof(int))) return 0;
            t->get_cursor((int*)arg[0], (int*)arg[1]); return 0;
        case SYSCALL_API_DRAW_BOX:         t->draw_box((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], arg[4], arg[5]); return 0;
        case SYSCALL_API_DRAW_BOX_TITLED:
            if (!user_str_ok((const char*)arg[4], 256)) return 0;
            t->draw_box_titled((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], (const char*)arg[4], arg[5], arg[6]); return 0;
        case SYSCALL_API_FILL_RECT:        t->fill_rect((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], (char)arg[4], arg[5], arg[6]); return 0;
        case SYSCALL_API_DRAW_HLINE:       t->draw_hline((int)arg[0], (int)arg[1], (int)arg[2], arg[3], arg[4]); return 0;
        case SYSCALL_API_DRAW_VLINE:       t->draw_vline((int)arg[0], (int)arg[1], (int)arg[2], arg[3], arg[4]); return 0;
        case SYSCALL_API_DRAW_WINDOW:
            if (!user_str_ok((const char*)arg[4], 256)) return 0;
            t->draw_window((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], (const char*)arg[4]); return 0;
        case SYSCALL_API_DRAW_BUTTON:
            if (!user_str_ok((const char*)arg[2], 256)) return 0;
            t->draw_button((int)arg[0], (int)arg[1], (const char*)arg[2], (int)arg[3]); return 0;
        case SYSCALL_API_DRAW_INPUT:
            if (!user_str_ok((const char*)arg[3], 256)) return 0;
            t->draw_input((int)arg[0], (int)arg[1], (int)arg[2], (const char*)arg[3], (int)arg[4]); return 0;
        case SYSCALL_API_DRAW_PROGRESS:    t->draw_progress((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3]); return 0;
        case SYSCALL_API_STR_COMPARE:
            if (!user_str_ok((const char*)arg[0], 1024) || !user_str_ok((const char*)arg[1], 1024)) return 0;
            return (uint32_t)t->str_compare((const char*)arg[0], (const char*)arg[1]);
        case SYSCALL_API_STR_LENGTH:
            if (!user_str_ok((const char*)arg[0], 1024)) return 0;
            return (uint32_t)t->str_length((const char*)arg[0]);
        case SYSCALL_API_STR_COPY:
            if (!user_ptr_ok((void*)arg[0], 256) || !user_str_ok((const char*)arg[1], 256)) return 0;
            t->str_copy((char*)arg[0], (const char*)arg[1]); return 0;
        case SYSCALL_API_MEM_COPY:
            if (!user_ptr_ok((void*)arg[0], arg[2]) || !user_ptr_ok((const void*)arg[1], arg[2])) return 0;
            t->mem_copy((void*)arg[0], (const void*)arg[1], arg[2]); return 0;
        case SYSCALL_API_MEM_SET:
            if (!user_ptr_ok((void*)arg[0], arg[2])) return 0;
            t->mem_set((void*)arg[0], (uint8_t)arg[1], arg[2]); return 0;
        case SYSCALL_API_GET_TIME:
            if (!user_ptr_ok((void*)arg[0], 1) || !user_ptr_ok((void*)arg[1], 1) || !user_ptr_ok((void*)arg[2], 1)) return 0;
            t->get_time((uint8_t*)arg[0], (uint8_t*)arg[1], (uint8_t*)arg[2]); return 0;
        case SYSCALL_API_GET_DATE:
            if (!user_ptr_ok((void*)arg[0], 1) || !user_ptr_ok((void*)arg[1], 1) || !user_ptr_ok((void*)arg[2], 2)) return 0;
            t->get_date((uint8_t*)arg[0], (uint8_t*)arg[1], (uint16_t*)arg[2]); return 0;
        case SYSCALL_API_DELAY:            t->delay(arg[0]); return 0;
        case SYSCALL_API_FILE_EXISTS:
            if (!user_str_ok((const char*)arg[0], 256)) return (uint32_t)-1;
            return (uint32_t)t->file_exists((const char*)arg[0]);
        case SYSCALL_API_FILE_READ:
            if (!user_str_ok((const char*)arg[0], 256) || !user_ptr_ok((void*)arg[1], arg[2])) return (uint32_t)-1;
            return (uint32_t)t->file_read((const char*)arg[0], (void*)arg[1], arg[2]);
        case SYSCALL_API_FILE_SIZE:
            if (!user_str_ok((const char*)arg[0], 256)) return 0;
            return t->file_size((const char*)arg[0]);
        case SYSCALL_API_FILE_WRITE:
            if (!user_str_ok((const char*)arg[0], 256) || !user_ptr_ok((const void*)arg[1], arg[2])) return (uint32_t)-1;
            return (uint32_t)t->file_write((const char*)arg[0], (const void*)arg[1], arg[2]);
        case SYSCALL_API_HAS_KEY:          return (uint32_t)t->has_key();
        case SYSCALL_API_GET_KEY:          return (uint32_t)(uint8_t)t->get_key();
        case SYSCALL_API_CLEAR_KEYS:       t->clear_keys(); return 0;
        case SYSCALL_API_GET_SYSINFO:
            if (!user_ptr_ok((void*)arg[0], sizeof(sys_info_t))) return 0;
            t->get_sysinfo((sys_info_t*)arg[0]); return 0;
        case SYSCALL_API_SET_USERNAME:
            if (!user_str_ok((const char*)arg[0], 32)) return 0;
            t->set_username((const char*)arg[0]); return 0;
        case SYSCALL_API_SET_HOSTNAME:
            if (!user_str_ok((const char*)arg[0], 32)) return 0;
            t->set_hostname((const char*)arg[0]); return 0;
        case SYSCALL_API_GFX_PIXEL:        t->gfx_pixel((int)arg[0], (int)arg[1], arg[2]); return 0;
        case SYSCALL_API_GFX_LINE:         t->gfx_line((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], arg[4]); return 0;
        case SYSCALL_API_GFX_RECT:         t->gfx_rect((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], arg[4]); return 0;
        case SYSCALL_API_GFX_FILLRECT:     t->gfx_fillrect((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], arg[4]); return 0;
        case SYSCALL_API_GFX_CIRCLE:       t->gfx_circle((int)arg[0], (int)arg[1], (int)arg[2], arg[3]); return 0;
        case SYSCALL_API_GFX_FILLCIRCLE:   t->gfx_fillcircle((int)arg[0], (int)arg[1], (int)arg[2], arg[3]); return 0;
        case SYSCALL_API_GFX_CHAR:         t->gfx_char((int)arg[0], (int)arg[1], (char)arg[2], arg[3], arg[4]); return 0;
        case SYSCALL_API_GFX_TEXT:
            if (!user_str_ok((const char*)arg[2], 1024)) return 0;
            t->gfx_text((int)arg[0], (int)arg[1], (const char*)arg[2], arg[3], arg[4]); return 0;
        case SYSCALL_API_GFX_WIDTH:        return (uint32_t)t->gfx_width();
        case SYSCALL_API_GFX_HEIGHT:       return (uint32_t)t->gfx_height();
        case SYSCALL_API_GFX_BLIT:
            if (!user_ptr_ok((const void*)arg[4], arg[2] * arg[3] * sizeof(uint32_t))) return 0;
            t->gfx_blit((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], (const uint32_t*)arg[4]); return 0;
        case SYSCALL_API_GUI_LABEL:
            if (!user_str_ok((const char*)arg[2], 1024)) return 0;
            t->gui_label((int)arg[0], (int)arg[1], (const char*)arg[2], arg[3]); return 0;
        case SYSCALL_API_GUI_CHECKBOX:
            if (!user_str_ok((const char*)arg[2], 1024)) return 0;
            t->gui_checkbox((int)arg[0], (int)arg[1], (const char*)arg[2], (int)arg[3]); return 0;
        case SYSCALL_API_GUI_MENUBAR:      t->gui_menubar((int)arg[0], (const char**)arg[1], (int)arg[2], (int)arg[3]); return 0;
        case SYSCALL_API_GUI_TABBAR:       t->gui_tabbar((int)arg[0], (int)arg[1], (int)arg[2], (const char**)arg[3], (int)arg[4], (int)arg[5]); return 0;
        case SYSCALL_API_GUI_DRAW_CURSOR:  t->gui_draw_cursor((int)arg[0], (int)arg[1]); return 0;
        case SYSCALL_API_GUI_LISTBOX:      t->gui_listbox((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], (const char**)arg[4], (int)arg[5], (int)arg[6]); return 0;
        case SYSCALL_API_GUI_SCROLLBAR:    t->gui_scrollbar((int)arg[0], (int)arg[1], (int)arg[2], (int)arg[3], (int)arg[4], (int)arg[5]); return 0;
        case SYSCALL_API_GUI_POPUP_MENU:   t->gui_popup_menu((int)arg[0], (int)arg[1], (const char**)arg[2], (int)arg[3], (int)arg[4]); return 0;
        case SYSCALL_API_GUI_TITLEBAR:
            if (!user_str_ok((const char*)arg[3], 1024)) return 0;
            t->gui_titlebar((int)arg[0], (int)arg[1], (int)arg[2], (const char*)arg[3], (int)arg[4]); return 0;
        case SYSCALL_API_STR_CONCAT:
            if (!user_ptr_ok((void*)arg[0], 256) || !user_str_ok((const char*)arg[1], 256)) return 0;
            t->str_concat((char*)arg[0], (const char*)arg[1]); return 0;
        case SYSCALL_API_STR_NCOMPARE:
            if (!user_str_ok((const char*)arg[0], 1024) || !user_str_ok((const char*)arg[1], 1024)) return 0;
            return (uint32_t)t->str_ncompare((const char*)arg[0], (const char*)arg[1], (int)arg[2]);
        case SYSCALL_API_STR_TO_INT:
            if (!user_str_ok((const char*)arg[0], 64)) return 0;
            return (uint32_t)t->str_to_int((const char*)arg[0]);
        case SYSCALL_API_INT_TO_STR:
            if (!user_ptr_ok((void*)arg[1], 16)) return 0;
            t->int_to_str((int)arg[0], (char*)arg[1]); return 0;
        case SYSCALL_API_MOUSE_GET_STATE:
            if (!user_ptr_ok((void*)arg[0], sizeof(mouse_state_t))) return 0;
            t->mouse_get_state((mouse_state_t*)arg[0]); return 0;
        case SYSCALL_API_MOUSE_SET_POS:    t->mouse_set_pos((int)arg[0], (int)arg[1]); return 0;
        case SYSCALL_API_MOUSE_SET_SENS:   t->mouse_set_sens((int)arg[0], (int)arg[1]); return 0;
        default:                           return 0;
    }
}

void tss_init(void) {
    idt_ptr_t gdtr;
    __asm__ volatile ("sgdt %0" : "=m"(gdtr));

    memset(&kernel_tss, 0, sizeof(kernel_tss));
    kernel_tss.ss0 = 0x10;
    kernel_tss.esp0 = 0x9D000;
    kernel_tss.iomap_base = sizeof(kernel_tss);

    gdt_entry_t* gdt = (gdt_entry_t*)(uintptr_t)gdtr.base;
    gdt_set_tss_descriptor(gdt, 5, (uint32_t)(uintptr_t)&kernel_tss, sizeof(kernel_tss) - 1);

    __asm__ volatile ("ltr %0" : : "r"((uint16_t)TSS_SELECTOR));
}