[BITS 32]

global _start
extern kernel_main

section .text

GDTR_SAVE   equ 0x6FF0
SAVED_ESP   equ 0x6FE0
SAVED_SS    equ 0x6FEC
RESULT_ADDR equ 0x6FEE
TRAMP_BASE  equ 0x5000

_start:
    mov esp, 0x90000
    ; копия трамплина в 0x5000
    mov esi, tramp_blob
    mov edi, TRAMP_BASE
    mov ecx, (tramp_blob_end - tramp_blob)
    rep movsb
    call kernel_main
.halt:
    cli
    hlt
    jmp .halt

global keyboard_handler
extern keyboard_callback

keyboard_handler:
    pusha
    xor eax, eax
    in  al, 0x60
    push eax
    call keyboard_callback
    add  esp, 4
    mov  al, 0x20
    out  0x20, al
    popa
    iret

global load_idt
load_idt:
    mov eax, [esp + 4]
    lidt [eax]
    ret

global enable_interrupts
enable_interrupts:
    sti
    ret

global disable_interrupts
disable_interrupts:
    cli
    ret

; Обработчики исключений CPU (INT 0-19)

extern exception_handler

%macro ISR_NO_ERR 1
global isr%1
isr%1:
    cli
    push dword 0
    push dword %1
    jmp isr_common
%endmacro

; Макрос для исключения С кодом ошибки
%macro ISR_ERR 1
global isr%1
isr%1:
    cli
    push dword %1
    jmp isr_common
%endmacro

ISR_NO_ERR 0
ISR_NO_ERR 1
ISR_NO_ERR 2
ISR_NO_ERR 3
ISR_NO_ERR 4
ISR_NO_ERR 5
ISR_NO_ERR 6
ISR_NO_ERR 7
ISR_ERR    8
ISR_NO_ERR 9
ISR_ERR    10
ISR_ERR    11
ISR_ERR    12
ISR_ERR    13
ISR_ERR    14
ISR_NO_ERR 15
ISR_NO_ERR 16
ISR_ERR    17
ISR_NO_ERR 18
ISR_NO_ERR 19

isr_common:
    ; Сохраняем регистры
    pusha
    push ds
    push es
    push fs
    push gs

    ; Загрузка сегментов ядра
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; Передача указателя на фрейм регистров
    push esp
    call exception_handler
    add esp, 4

    ; Восстанавливаем
    pop gs
    pop fs
    pop es
    pop ds
    popa
    add esp, 8 ; убираем int_num и err_code
    iret

global pm_to_rm_write ; просто прыгает в трамплин по 0x5000
pm_to_rm_write:
    mov eax, TRAMP_BASE
    jmp eax
tramp_blob:

[BITS 32]
.entry32:
    pusha
    pushf
    cli

    mov eax, [esp + 36]         ; lba
    mov [0x6010], eax

    movzx eax, byte [esp + 40]  ; drive
    mov [0x6014], al

    mov esi, [esp + 44]         ; buf - 0x6100
    mov edi, 0x6100
    mov ecx, 128
    rep movsd

    sgdt [GDTR_SAVE]
    mov  [SAVED_ESP], esp
    mov  ax, ss
    mov  [SAVED_SS], ax

    db 0xEA
    dd TRAMP_BASE + (.pm16 - tramp_blob)
    dw 0x08

[BITS 16]
.pm16:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    mov eax, cr0
    and al, 0xFE
    mov cr0, eax

    db 0xEA
    dw (TRAMP_BASE + (.rm16 - tramp_blob)) & 0xFFFF
    dw 0x0000

.rm16:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov sp, 0x6F00

    ; DAP в 0x6000
    mov word  [0x6000], 0x0010
    mov word  [0x6002], 0x0001
    mov word  [0x6004], 0x0100   ; offset буфера
    mov word  [0x6006], 0x0600   ; сегмент -> 0x0600*16 = 0x6000+0x100 = 0x6100
    mov eax,  [0x6010]
    mov dword [0x6008], eax
    mov dword [0x600C], 0

    mov dl, [0x6014]
    mov si, 0x6000
    mov ah, 0x43
    xor al, al
    int 0x13

    mov byte [0x6015], 0
    jnc .rm_ok
    mov byte [0x6015], 1
.rm_ok:

    ; A20
    in  al, 0x92
    or  al, 0x02
    and al, 0xFE
    out 0x92, al

    ; Восстанавливаем GDTR
    db 0x66         ; o32 prefix
    lgdt [GDTR_SAVE]

    ; Включаем PE
    mov eax, cr0
    or  al, 0x01
    mov cr0, eax

    ; Far jmp -> 32-bit PM с o32 префиксом
    db 0x66, 0xEA ; o32 far jmp
    dd TRAMP_BASE + (.pm32 - tramp_blob)
    dw 0x08

[BITS 32]
.pm32:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    movzx eax, word [SAVED_SS]
    mov ss, ax
    mov esp, [SAVED_ESP]

    xor eax, eax
    mov al, [0x6015]
    test al, al
    setnz al
    neg eax
    mov [RESULT_ADDR], eax

    popf
    popa
    mov eax, [RESULT_ADDR]
    ret

tramp_blob_end: