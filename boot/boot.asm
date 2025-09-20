; Zinux Boot Loader
; This is the first sector of the bootable floppy/hard disk.
; It runs in 16-bit real mode and loads the kernel.

org 0x7c00
bits 16

SECONDS_TO_WAIT    equ 5
TICKS_PER_SECOND   equ 18         ; ~18.2, using 18 for simplicity
TICKS_TO_WAIT      equ SECONDS_TO_WAIT * TICKS_PER_SECOND

; The kernel is located at sector 2 and loaded to address 0x10000
KERNEL_SECTOR_START equ 2
KERNEL_TARGET_ADDRESS equ 0x10000
KERNEL_SECTORS equ 2 ; Assuming kernel is ~1KB. Adjust as needed.

start:
    ; Use the drive number passed by the BIOS
    mov [boot_drive_id], dl     
    
    cli                         ; Disable interrupts during setup
    xor ax, ax                  ; AX = 0
    mov ds, ax                  ; DS = 0
    mov es, ax                  ; ES = 0
    mov ss, ax                  ; SS = 0
    mov sp, 0x7c00              ; Set up the stack at the end of the bootloader
    sti                         ; Re-enable interrupts

    call clear_screen

    mov si, menu_title
    call print_string
    mov si, option1
    call print_string
    mov si, option2
    call print_string
    mov si, prompt
    call print_string

    ; --- Get current ticks and calculate the 32-bit target (CX:DX) ---
    mov ah, 0x00
    int 0x1a                    ; CX:DX = ticks since midnight
    mov ax, dx
    add ax, TICKS_TO_WAIT
    adc cx, 0                   ; Add with carry for high word
    mov [target_low], ax
    mov [target_high], cx

    mov byte [last_second], 0FFh

countdown_loop:
    call update_timer_display

    ; Check for key press
    mov ah, 0x01
    int 0x16
    jz check_time
    mov ah, 0x00
    int 0x16
    cmp al, '1'
    je boot_kernel
    cmp al, '2'
    je reboot_system
    jmp countdown_loop

check_time:
    mov ah, 0x00
    int 0x1a                    ; CX:DX = current ticks

    ; Compare CX:DX (current) with target_high:target_low (target)
    mov ax, [target_high]
    cmp cx, ax
    jb countdown_loop           ; current_high < target_high => still waiting
    ja .time_reached            ; current_high > target_high => timeout
    mov ax, [target_low]
    cmp dx, ax
    jb countdown_loop
.time_reached:
    jmp boot_kernel

boot_kernel:
    call clear_screen
    mov si, kernel_msg
    call print_string
    
    ; Load the kernel from disk
    mov ax, KERNEL_TARGET_ADDRESS
    mov es, ax
    mov bx, 0                   ; ES:BX = 0x10000:0x0000
    mov al, KERNEL_SECTORS      ; Number of sectors to read
    mov cl, KERNEL_SECTOR_START
    mov dl, [boot_drive_id]     ; Load the drive ID here again to be safe
    call read_disk_sectors

    ; Jmp to 32-bit transition
    jmp protected_mode_transition

reboot_system:
    int 0x19

; -------------------------
; read_disk_sectors: Reads sectors from disk using INT 13h
; -------------------------
read_disk_sectors:
    pusha
    
    mov ah, 0x02                ; BIOS read sectors function
    mov ch, 0                   ; Cylinder 0
    mov dh, 0                   ; Head 0
    
    int 0x13
    
    jnc .done_read              ; If no carry flag, it's successful
    
    mov si, read_error_msg
    call print_string
    mov ah, 0
    int 0x16                    ; Wait for key press
    int 0x19                    ; Reboot
    
.done_read:
    popa
    ret

; -------------------------
; update_timer_display: Calculates and prints remaining seconds
; -------------------------
update_timer_display:
    pusha

    mov ah, 0x00
    int 0x1a                    ; CX:DX = current ticks

    ; remaining = target - current (32-bit: high:low)
    mov ax, [target_low]
    sub ax, dx
    mov bx, [target_high]
    sbb bx, cx                  ; BX:AX = remaining ticks
    
    mov dx, bx                  ; DX:AX = remaining (for div)
    mov cx, TICKS_PER_SECOND
    div cx                      ; AX = remaining / TICKS_PER_SECOND
    
    inc ax                      ; Count 5,4,3... instead of 4,3,2...
    cmp al, [last_second]
    je .skip_print
    mov byte [last_second], al

    ; Set cursor position (row 5, col 33) and print
    mov dh, 5
    mov dl, 33
    mov bh, 0
    mov ah, 0x02
    int 0x10

    add al, '0'
    mov ah, 0x0e
    int 0x10

.skip_print:
    popa
    ret

; -------------------------
; print_string: Prints a null-terminated string at DS:SI
; -------------------------
print_string:
    mov ah, 0x0e
.print_loop:
    lodsb
    cmp al, 0
    je .done
    int 0x10
    jmp .print_loop
.done:
    ret

clear_screen:
    mov ah, 0x00
    mov al, 0x03                ; Video mode 03h (text 80x25)
    int 0x10
    ret

; --- Data ---
menu_title      db '--- Zrub Boot Loader ---',13,10,13,10,0
option1         db '1. Boot Zinux Kernel',13,10,0
option2         db '2. Reboot (to enter BIOS)',13,10,0
prompt          db 'Auto-booting in...  seconds',13,10,0
kernel_msg      db 'Loading Zinux Kernel...',0
read_error_msg  db 'Disk Read Error!', 13, 10, 0

target_low      dw 0
target_high     dw 0

last_second     db 0
boot_drive_id   db 0

; --- GDT ---
gdt_start:
    dd 0x0, 0x0

gdt_code:
    dw 0xffff
    dw 0x0000
    db 0x00
    db 10011010b
    db 11001111b
    db 0x00

gdt_data:
    dw 0xffff
    dw 0x0000
    db 0x00
    db 10010010b
    db 11001111b
    db 0x00

gdt_end:

gdt_pointer:
    dw gdt_end - gdt_start - 1
    dd gdt_start

code_segment equ gdt_code - gdt_start
data_segment equ gdt_data - gdt_start

; --- reboot to protected mode ---
protected_mode_transition:
    cli
    lgdt [gdt_pointer]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp dword code_segment:protected_mode_start

[bits 32]
protected_mode_start:
    mov ax, data_segment
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    jmp KERNEL_TARGET_ADDRESS

; --- 510 byte ---
times 510-($-$$) db 0
dw 0xAA55
