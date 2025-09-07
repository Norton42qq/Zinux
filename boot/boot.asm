; boot.asm — MBR bootloader for Zinux
; Loads stage2 from LBA 1 into 0x8000
BITS 16
org 0x7c00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00
    sti

    ; Save boot drive
    mov [boot_drive], dl

    ; Debug: Confirm MBR loaded
    mov si, msg_mbr
    call print

    ; Reset disk system
    mov si, msg_reset
    call print
    xor ah, ah
    mov dl, [boot_drive]
    int 0x13
    jc disk_error

    ; Setup DAP for stage2 (1 sector to test)
    mov si, msg_dap
    call print
    mov byte [dap], 16
    mov byte [dap+1], 0
    mov word [dap+2], 1       ; Read 1 sector
    mov word [dap+4], 0
    mov word [dap+6], 0x800   ; Load at 0x8000 (0x800:0)
    mov dword [dap+8], 1      ; LBA 1
    mov dword [dap+12], 0

    ; Read stage2
    mov si, msg_read
    call print
    mov dl, [boot_drive]
    mov si, dap
    mov ah, 0x42
    int 0x13
    jc disk_error

    ; Jump to stage2
    mov si, msg_jump
    call print
    jmp 0x8000

disk_error:
    mov si, msg_error
    call print
    jmp hang

print:
    pusha
    mov ah, 0x0E
.print_loop:
    lodsb
    test al, al
    jz .done
    int 0x10
    jmp .print_loop
.done:
    popa
    ret

hang:
    hlt
    jmp hang

; Data
boot_drive db 0
msg_mbr db "MBR loaded!", 0x0D, 0x0A, 0
msg_reset db "Resetting disk...", 0x0D, 0x0A, 0
msg_dap db "Setting up DAP...", 0x0D, 0x0A, 0
msg_read db "Reading stage2...", 0x0D, 0x0A, 0
msg_jump db "Jumping to stage2...", 0x0D, 0x0A, 0
msg_error db "Disk error!", 0x0D, 0x0A, 0
dap: times 16 db 0

; Boot signature
times 510-($-$$) db 0
dw 0xAA55