[BITS 16]
[ORG 0x7C00]

STAGE2_OFFSET   equ 0x7E00
STAGE2_SECTORS  equ 8

start:
    ; Инициализация сегментов
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    sti
    
    mov [boot_drive], dl
    
    ; Очистка экрана
    mov ax, 0x0003
    int 0x10
    
    ; Приветствие
    mov si, msg_boot
    call print
    
    ; Проверка CPU
    mov si, msg_cpu
    call print
    call check_cpu
    jc error
    mov si, msg_ok
    call print
    
    ; Загрузка Stage 2
    mov si, msg_load
    call print
    call load_stage2
    jc error
    mov si, msg_ok
    call print
    
    ; Переход к Stage 2
    jmp STAGE2_OFFSET

;--------------------
; Проверка CPU 386+
;--------------------
check_cpu:
    pushf
    pop ax
    mov cx, ax
    xor ax, 0x4000
    push ax
    popf
    pushf
    pop ax
    push cx
    popf
    xor ax, cx
    jz .fail
    clc
    ret
.fail:
    stc
    ret

;--------------------
; Загрузка Stage 2
;--------------------
load_stage2:
    mov ah, 0x02
    mov al, STAGE2_SECTORS
    mov ch, 0
    mov cl, 2
    mov dh, 0
    mov dl, [boot_drive]
    mov bx, STAGE2_OFFSET
    int 0x13
    jc .fail
    cmp al, STAGE2_SECTORS
    jne .fail
    clc
    ret
.fail:
    stc
    ret

;--------------------
; Ошибка
;--------------------
error:
    mov si, msg_err
    call print
.halt:
    hlt
    jmp .halt

;--------------------
; Вывод строки
;--------------------
print:
    lodsb
    or al, al
    jz .done
    mov ah, 0x0E
    mov bh, 0
    int 0x10
    jmp print
.done:
    ret

;--------------------
; Данные
;--------------------
boot_drive: db 0

msg_boot:   db '=== Zrub Boot Loader ===', 13, 10, 0
msg_cpu:    db 'Checking CPU... ', 0
msg_load:   db 'Loading Stage2... ', 0
msg_ok:     db 'OK', 13, 10, 0
msg_err:    db 'FAIL', 13, 10, 'System halted', 0

times 510-($-$$) db 0
dw 0xAA55