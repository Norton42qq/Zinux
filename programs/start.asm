[BITS 32]

global _start
extern kernel_main

section .text

; ══════════════════════════════════════════════════════════════
; Константы временной области в нижней памяти (0x6000-0x6FFF)
; ══════════════════════════════════════════════════════════════
GDTR_SAVE   equ 0x6FF0      ; 6 байт — сохранённый GDTR
SAVED_ESP   equ 0x6FE0      ; 4 байта
SAVED_SS    equ 0x6FEC      ; 2 байта
RESULT_ADDR equ 0x6FEE      ; 4 байта

; Трамплин копируется сюда при старте
TRAMP_BASE  equ 0x5000

_start:
    mov esp, 0x90000

    ; Копируем трамплин в 0x5000 (нижняя память, свободна)
    mov esi, tramp_blob
    mov edi, TRAMP_BASE
    mov ecx, (tramp_blob_end - tramp_blob)
    rep movsb

    call kernel_main
.halt:
    cli
    hlt
    jmp .halt

; ── Обработчик клавиатуры IRQ1 ──
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

; ── IDT ──
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

; ══════════════════════════════════════════════════════════════
; pm_to_rm_write(uint32_t lba, uint8_t drive, void* buf)
;
; Вызывает трамплин скопированный в TRAMP_BASE=0x5000.
; Трамплин — position-independent blob: он знает свой базовый
; адрес и строит far jmp с абсолютными адресами 0x5000+offset.
; ══════════════════════════════════════════════════════════════

global pm_to_rm_write
pm_to_rm_write:
    jmp TRAMP_BASE          ; прыгаем в скопированный трамплин

; ── Blob трамплина (исполняется по адресу TRAMP_BASE) ──
; Написан как position-independent: все адреса вычисляются
; относительно TRAMP_BASE, а не реального адреса в .text.
;
; Структура blob:
;   offset +0x00 : [BITS 32] — сохранить args, copy sector, sgdt, jmp→pm16
;   offset +0x50 : [BITS 16] — сбросить PE, jmp→rm16
;   offset +0x60 : [BITS 16] — RM: DAP, INT13h, вернуться в PM
;   offset +0xA0 : [BITS 32] — восстановить состояние, вернуть результат

tramp_blob:

; ─────────────────────────────────────────────
; [BITS 32] @ TRAMP_BASE+0x00
; ─────────────────────────────────────────────
[BITS 32]
.b32_start:
    pusha
    pushf
    cli

    mov eax, [esp + 36]         ; lba
    mov [0x6010], eax

    movzx eax, byte [esp + 40]  ; drive
    mov [0x6014], al

    mov esi, [esp + 44]         ; buf → 0x6100
    mov edi, 0x6100
    mov ecx, 128
    rep movsd

    sgdt [GDTR_SAVE]
    mov  [SAVED_ESP], esp
    mov  ax, ss
    mov  [SAVED_SS], ax

    ; far jmp в 16-bit PM
    ; Адрес: TRAMP_BASE + (.b16_pm16 - tramp_blob)
    ; Кодируем вручную: opcode EA, dd offset32, dw selector
    db 0xEA
    dd TRAMP_BASE + (.b16_pm16 - tramp_blob)
    dw 0x08

; ─────────────────────────────────────────────
; [BITS 16] @ TRAMP_BASE + (.b16_pm16 - tramp_blob)
; Ещё в protected mode, но уже 16-bit операнды
; ─────────────────────────────────────────────
[BITS 16]
.b16_pm16:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    mov eax, cr0
    and al, 0xFE
    mov cr0, eax

    ; far jmp → real mode, CS=0
    db 0xEA
    dw TRAMP_BASE + (.b16_rm - tramp_blob)
    dw 0x0000

; ─────────────────────────────────────────────
; [BITS 16] Real mode
; ─────────────────────────────────────────────
.b16_rm:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov sp, 0x6F00

    ; Строим DAP в 0x6000
    mov word  [0x6000], 0x0010
    mov word  [0x6002], 0x0001
    mov word  [0x6004], 0x0100   ; buf offset
    mov word  [0x6006], 0x0600   ; buf seg → линейный 0x6100
    mov eax,  [0x6010]
    mov dword [0x6008], eax
    mov dword [0x600C], 0

    ; INT 13h Extended Write
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

    ; Восстанавливаем GDTR и включаем PM
    lgdt [GDTR_SAVE]
    mov eax, cr0
    or  al, 0x01
    mov cr0, eax

    ; far jmp → 32-bit PM
    db 0xEA
    dd TRAMP_BASE + (.b32_pm32 - tramp_blob)
    dw 0x08

; ─────────────────────────────────────────────
; [BITS 32] Возврат
; ─────────────────────────────────────────────
[BITS 32]
.b32_pm32:
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