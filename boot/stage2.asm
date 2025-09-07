; stage2.asm — Zinux second-stage bootloader (Real → Protected → Long mode)
; ---------------------------------------------------------
BITS 16
org 0x8000

start16:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x9000
    sti

    mov [boot_drive], dl

    ; Debug: Confirm stage2 loaded
    mov si, msg_start
    call print

; -----------------------------
; Step1: Skip A20 (QEMU enables it)
; -----------------------------
    mov si, step1
    call print

; -----------------------------
; Step2: Load kernel
; -----------------------------
    mov si, step2
    call print

    ; Reset disk system
    mov si, msg_disk_reset
    call print
    mov dl, [boot_drive]
    xor ah, ah
    int 0x13
    jc disk_fail

    ; Setup DAP
    mov si, msg_dap_setup
    call print
    mov byte [dap], 16
    mov byte [dap+1], 0
    mov word [dap+2], 128
    mov word [dap+4], 0
    mov word [dap+6], 0x1000
    mov dword [dap+8], 65
    mov dword [dap+12], 0

    ; Read kernel
    mov si, msg_disk_read
    call print
    mov dl, [boot_drive]
    mov si, dap
    mov ah, 0x42
    int 0x13
    jc disk_fail

    ; Verify kernel magic
    mov si, msg_verify_magic
    call print
    mov eax, [0x100000]
    cmp eax, 0x12345678
    jne disk_fail

; -----------------------------
; Step3: Kernel loaded
; -----------------------------
    mov si, step3
    call print

; -----------------------------
; Protected Mode
; -----------------------------
    mov si, msg_protected
    call print
    cli
    lgdt [gdt_descriptor]

    mov eax, cr0
    or eax, 1
    mov cr0, eax

    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    jmp 0x08:protected_mode_entry

disk_fail:
    mov si, msg_diskerr
    call print
    jmp hang

hang:
    hlt
    jmp hang

; -----------------------------
; Print routine (16-bit)
; -----------------------------
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

; -----------------------------
; Protected Mode Entry (32-bit)
; -----------------------------
[BITS 32]
protected_mode_entry:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x80000

    mov esi, msg_protected32
    call print32

    xor eax, eax
    mov edi, 0x70000
    mov ecx, 0x3000/4
    rep stosd

    mov edi, 0x70000
    mov eax, 0x71000 | 0x003
    mov [edi], eax

    mov edi, 0x71000
    mov eax, 0x72000 | 0x003
    mov [edi], eax

    mov edi, 0x72000
    mov eax, 0x00000000 | 0x083
    mov [edi], eax
    mov eax, 0x00200000 | 0x083
    mov [edi+8], eax

    mov eax, [0x70000]
    cmp eax, 0x71000 | 0x003
    jne .page_fail
    mov eax, [0x71000]
    cmp eax, 0x72000 | 0x003
    jne .page_fail
    mov eax, [0x72000]
    cmp eax, 0x00000000 | 0x083
    jne .page_fail
    mov eax, [0x72000+8]
    cmp eax, 0x00200000 | 0x083
    jne .page_fail

    mov eax, 0x70000
    mov cr3, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    lgdt [gdt64_descriptor]
    jmp 0x08:long_mode_entry

.page_fail:
    mov eax, cr0
    and eax, ~1
    mov cr0, eax
    jmp 0x0000:.real_mode

[BITS 16]
.real_mode:
    mov ax, 0
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x9000
    mov si, msg_pageerr
    call print
    hlt
    jmp $

; -----------------------------
; Long Mode Entry (64-bit)
; -----------------------------
[BITS 64]
long_mode_entry:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov rsp, 0x200000

    mov word [0xB8000], 0x074C  ; 'L'
    mov word [0xB8002], 0x074D  ; 'M'
    mov word [0xB8004], 0x073A  ; ':'
    mov word [0xB8006], 0x074B  ; 'K'

    mov rax, 0x100000
    jmp rax

; -----------------------------
; Print routine (32-bit)
; -----------------------------
[BITS 32]
print32:
    push eax
    push ebx
    mov ebx, 0xB8000
.print_loop:
    lodsb
    test al, al
    jz .done
    mov [ebx], al
    mov byte [ebx+1], 0x07
    add ebx, 2
    jmp .print_loop
.done:
    pop ebx
    pop eax
    ret

; -----------------------------
; Data
; -----------------------------
boot_drive db 0
msg_start db "Stage2 started!", 0x0D, 0x0A, 0
step1 db "Step1: Skipping A20 (QEMU default)", 0x0D, 0x0A, 0
step2 db "Step2: Reading kernel...", 0x0D, 0x0A, 0
step3 db "Step3: Kernel loaded!", 0x0D, 0x0A, 0
msg_diskerr db "Disk error!", 0x0D, 0x0A, 0
msg_pageerr db "Page table setup failed!", 0x0D, 0x0A, 0
msg_protected db "Entering protected mode...", 0x0D, 0x0A, 0
msg_protected32 db "In protected mode!", 0
msg_disk_reset db "Resetting disk system...", 0x0D, 0x0A, 0
msg_dap_setup db "Setting up DAP...", 0x0D, 0x0A, 0
msg_disk_read db "Reading disk...", 0x0D, 0x0A, 0
msg_verify_magic db "Verifying kernel magic...", 0x0D, 0x0A, 0

dap: times 16 db 0

; -----------------------------
; GDT для защищенного режима
; -----------------------------
align 8
gdt_start:
    dq 0x0000000000000000
gdt_code32:
    dq 0x00CF9A000000FFFF
gdt_data32:
    dq 0x00CF92000000FFFF
gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

; -----------------------------
; GDT для длинного режима
; -----------------------------
align 8
gdt64_start:
    dq 0x0000000000000000
gdt64_code:
    dq 0x00209A0000000000
gdt64_data:
    dq 0x0000920000000000
gdt64_end:

gdt64_descriptor:
    dw gdt64_end - gdt64_start - 1
    dd gdt64_start