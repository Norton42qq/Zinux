/* Порт kernel/kernel_entry.asm — синтаксис GAS Intel (LLVM integrated as).
 *
 * Трамплин (0x5000) НЕ пересобирается здесь: его готовые 258 байт лежат в
 * tramp.bin (nasm) и копируются из статики zcore::arch::entry::TRAMP_BLOB
 * в 0x5000 на старте. Все внутренние far-jump-иммедиаты в блобе уже
 * привязаны к 0x5000, поэтому переезд на 0x5000 ничего не требует.
 */

.intel_syntax noprefix

/* ---- Константы (equ из kernel_entry.asm) ---- */
.equ GDTR_SAVE,      0x6FF0
.equ SAVED_ESP,      0x6FE0
.equ SAVED_SS,       0x6FEC
.equ RESULT_ADDR,    0x6FEE
.equ TRAMP_BASE,     0x5000
.equ TRAMP_LEN,      258
.equ USER_CODE_SEL,  0x1A
.equ USER_DATA_SEL,  0x22

/* ---- _start: первая функция в .text.zinux_entry ---- */
.section .text.zinux_entry, "ax", @progbits

.global _start
.extern kernel_main
.extern tramp_blob

_start:
    mov esp, 0x90000
    /* копия трамплина в 0x5000 */
    mov esi, OFFSET tramp_blob
    mov edi, TRAMP_BASE
    mov ecx, TRAMP_LEN
    rep movsb
    call kernel_main
.halt:
    cli
    hlt
    jmp .halt

/* ---- Синхронный вывод в консоль (INT 0x80, EAX=1) ---- */
.global keyboard_handler
.extern keyboard_callback

keyboard_handler:
    pusha
    xor eax, eax
    in al, 0x60
    push eax
    call keyboard_callback
    add esp, 4
    mov al, 0x20
    out 0x20, al
    popa
.byte 0xcf  /* iretd */

.global timer_handler
.extern pit_tick

timer_handler:
    pusha
    call pit_tick
    mov al, 0x20
    out 0x20, al
    popa
.byte 0xcf  /* iretd */

.global syscall_handler
.extern syscall_putc
.extern syscall_api_call

syscall_handler:
    mov edx, eax
    pusha
    mov eax, ds
    push eax
    mov eax, es
    push eax
    mov eax, fs
    push eax
    mov eax, gs
    push eax

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    cmp edx, 1
    je .sys_putc
    cmp edx, 2
    je .sys_exit
    cmp edx, 3
    je .sys_api
    jmp .done

.sys_putc:
    push ebx
    call syscall_putc
    add esp, 4
    xor eax, eax
    jmp .done

.sys_exit:
    mov dword ptr [saved_user_exit_code], ebx
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov esp, dword ptr [saved_kernel_esp]
    jmp dword ptr [saved_kernel_eip]

.sys_api:
    push ecx
    push ebx
    call syscall_api_call
    add esp, 8
    jmp .done

.done:
    mov [esp + 44], eax
    add esp, 16
    popa
.byte 0xcf  /* iretd */

.global mouse_irq_handler
.extern mouse_handler

mouse_irq_handler:
    pusha
    push 0
    call mouse_handler
    add esp, 4
    mov al, 0x20
    out 0xA0, al
    mov al, 0x20
    out 0x20, al
    popa
.byte 0xcf  /* iretd */

/* ---- Простые хелперы (вызываются из Rust) ---- */
.global load_idt
load_idt:
    mov eax, dword ptr [esp + 4]
    lidt [eax]
    ret

.global enable_interrupts
enable_interrupts:
    sti
    ret

.global disable_interrupts
disable_interrupts:
    cli
    ret

/* ---- Запуск/прерывание пользовательской программы ---- */
.global run_user_program
.global abort_user_program

run_user_program:
    cli
    mov dword ptr [saved_user_exit_code], 0
    mov dword ptr [saved_kernel_esp], esp
    mov dword ptr [saved_kernel_eip], OFFSET .return_from_user_program

    mov esi, dword ptr [esp + 4]
    mov ecx, dword ptr [esp + 8]

    mov ax, USER_DATA_SEL
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    push USER_DATA_SEL
    push ecx
    pushfd
    pop eax
    or eax, 0x00000200
    push eax
    push USER_CODE_SEL
    push esi
.byte 0xcf  /* iretd */

.user_ret_trampoline:
    mov ebx, eax
    mov eax, 2
    int 0x80
.ret_hang:
    jmp .ret_hang

.return_from_user_program:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    sti
    mov eax, dword ptr [saved_user_exit_code]
    ret

abort_user_program:
    cli
    mov eax, dword ptr [esp + 4]
    mov dword ptr [saved_user_exit_code], eax
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov esp, dword ptr [saved_kernel_esp]
    jmp dword ptr [saved_kernel_eip]

/* ---- ISR 0-19 (макросы развёрнуты вручную) ---- */
.extern exception_handler

.global isr0
isr0:
    cli
    push 0
    push 0
    jmp isr_common
.global isr1
isr1:
    cli
    push 0
    push 1
    jmp isr_common
.global isr2
isr2:
    cli
    push 0
    push 2
    jmp isr_common
.global isr3
isr3:
    cli
    push 0
    push 3
    jmp isr_common
.global isr4
isr4:
    cli
    push 0
    push 4
    jmp isr_common
.global isr5
isr5:
    cli
    push 0
    push 5
    jmp isr_common
.global isr6
isr6:
    cli
    push 0
    push 6
    jmp isr_common
.global isr7
isr7:
    cli
    push 0
    push 7
    jmp isr_common
.global isr8
isr8:
    cli
    push 8
    jmp isr_common
.global isr9
isr9:
    cli
    push 0
    push 9
    jmp isr_common
.global isr10
isr10:
    cli
    push 10
    jmp isr_common
.global isr11
isr11:
    cli
    push 11
    jmp isr_common
.global isr12
isr12:
    cli
    push 12
    jmp isr_common
.global isr13
isr13:
    cli
    push 13
    jmp isr_common
.global isr14
isr14:
    cli
    push 14
    jmp isr_common
.global isr15
isr15:
    cli
    push 0
    push 15
    jmp isr_common
.global isr16
isr16:
    cli
    push 0
    push 16
    jmp isr_common
.global isr17
isr17:
    cli
    push 17
    jmp isr_common
.global isr18
isr18:
    cli
    push 0
    push 18
    jmp isr_common
.global isr19
isr19:
    cli
    push 0
    push 19
    jmp isr_common

isr_common:
    pusha
    /* Пуш 32-битных сегментных регистров (на LLVM as `push ds` кодируется
       как 16-битный pushw, что ломает фрейм registers_t). */
    mov eax, ds
    push eax
    mov eax, es
    push eax
    mov eax, fs
    push eax
    mov eax, gs
    push eax

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    push esp
    call exception_handler
    add esp, 4

    /* Сегменты всё равно 0x10 — просто пропустить 4 сохранённых dword. */
    add esp, 16
    popa
    add esp, 8
.byte 0xcf  /* iretd */

/* ---- Переход в реальный режим: просто прыжок в трамплин 0x5000 ---- */
.global pm_to_rm_write
pm_to_rm_write:
    mov eax, TRAMP_BASE
    jmp eax

/* ---- Глобальные переменные (section .bss) ---- */
.section .bss
saved_kernel_esp:   .zero 4
saved_kernel_eip:   .zero 4
saved_user_exit_code: .zero 4