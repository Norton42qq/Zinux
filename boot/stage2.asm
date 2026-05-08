[BITS 16]
[ORG 0x7E00]

KERNEL_OFFSET   equ 0x1000
LFB_STORE       equ 0x8000
VIDEO_INFO_ADDR equ 0x8000
DISK_BUF_FLAG   equ 0x8014
DISK_BUF_DRIVE  equ 0x8018

FAT_LBA_START   equ 2048
FAT_SECTORS     equ 1024

FAT_BUF_SEG     equ 0x5000
FAT_BUF_OFF     equ 0x0000

jmp stage2_start

;--------------------
; Данные
;--------------------
selected:       db 0
boot_drive:     db 0x80

title_txt:      db '======| Zrub Boot Loader |======', 0
menu1_txt:      db '  [1] Load Zinux                ', 0
menu2_txt:      db '  [2] Reboot PC                 ', 0
hint_txt:       db 'Use 1/2 or arrows, Enter=select ', 0
loading_txt:    db 'Loading kernel... ', 0
disk_txt:       db 'Loading disk...   ', 0
vesa_txt:       db 'Setting video...   ', 0
ok_txt:         db 'OK', 0
skip_txt:       db 'SKIP', 0
err_txt:        db 'ERROR', 0
reboot_txt:     db 'Rebouting...', 0
vesa_fail_txt:  db 'video init failed! Halting.', 13, 10, 0

vbe_ctrl_info:  times 512 db 0
vbe_mode_info:  times 256 db 0

; --- Таблица разрешений экрана ---
mode_wishlist:
    dw 1920, 1080, 32
    dw 1920, 1080, 24
    dw 1920, 1080, 16
    dw 1600,  900, 32
    dw 1600,  900, 16
    dw 1440, 1050, 32
    dw 1440, 1050, 16
    dw 1440,  900, 32
    dw 1440,  900, 16
    dw 1280,  720, 32
    dw 1280,  720, 24
    dw 1280,  720, 16
    dw 1024,  768, 32
    dw 1024,  768, 16
    dw  800,  600, 32
    dw  800,  600, 16
    dw  800,  600,  8
    dw  640,  480, 32
    dw  640,  480, 16
    dw  640,  480,  8
    dw 0

vi_found_mode:  dw 0
vi_wish_w:      dw 0
vi_wish_h:      dw 0
vi_wish_bpp:    dw 0
vi_modelist_ptr: dd 0

align 4
dap:
    db 0x10
    db 0
    dw 64
    dw FAT_BUF_OFF
    dw FAT_BUF_SEG
    dq FAT_LBA_START

;--------------------
; Точка входа
;--------------------
stage2_start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00
    mov [boot_drive], dl
    mov dword [LFB_STORE], 0
    mov dword [DISK_BUF_FLAG], 0
    sti
    jmp main_loop

;--------------------
; Главное меню
;--------------------
main_loop:
    call draw_menu
    call get_key
    cmp al, '1'
    je do_boot
    cmp al, '2'
    je do_reboot
    cmp ah, 0x48
    je key_up
    cmp ah, 0x50
    je key_down
    cmp al, 13
    je do_select
    jmp main_loop

key_up:
    mov byte [selected], 0
    jmp main_loop
key_down:
    mov byte [selected], 1
    jmp main_loop

do_select:
    cmp byte [selected], 0
    je do_boot
    jmp do_reboot

;--------------------
; Загрузка системы
;--------------------
do_boot:
    call clear_screen

    ; 1. Загрузка ядра
    mov dh, 9
    mov dl, 24
    mov si, loading_txt
    call print_at
    call load_kernel
    jc .kern_err
    mov dh, 9
    mov dl, 42
    mov si, ok_txt
    mov bl, 0x0A
    call print_at_color
    jmp .do_disk

.kern_err:
    mov dh, 9
    mov dl, 42
    mov si, err_txt
    mov bl, 0x0C
    call print_at_color
    call get_key
    jmp main_loop

.do_disk:
    ; 2. Чтение FAT16 раздел в RAM
    mov dh, 10
    mov dl, 24
    mov si, disk_txt
    call print_at
    call load_fat_to_ram
    mov dh, 10
    mov dl, 42
    cmp dword [DISK_BUF_FLAG], 0
    je .disk_skip
    mov si, ok_txt
    mov bl, 0x0A
    call print_at_color
    jmp .do_vesa
.disk_skip:
    mov si, skip_txt
    mov bl, 0x0E
    call print_at_color

.do_vesa:
    ; 3. VESA
    mov dh, 11
    mov dl, 24
    mov si, vesa_txt
    call print_at
    call vesa_init
    jc .vesa_fail
    mov dh, 11
    mov dl, 42
    mov si, ok_txt
    mov bl, 0x0A
    call print_at_color

    ; Пауза чтобы увидеть статус
    mov cx, 0x8000
.pause: nop
    loop .pause

    jmp enter_pm

.vesa_fail:
    mov dh, 11
    mov dl, 42
    mov si, err_txt
    mov bl, 0x0C
    call print_at_color
    mov cx, 0x8000
.pause2: nop
    loop .pause2
    jmp enter_pm

;------------------------------
; Загрузка FAT16 раздела в RAM
;------------------------------
load_fat_to_ram:
    mov ah, 0x41
    mov bx, 0x55AA
    mov dl, [boot_drive]
    int 0x13
    jc .fail

    mov word  [dap+2],  64
    mov word  [dap+4],  FAT_BUF_OFF
    mov word  [dap+6],  FAT_BUF_SEG
    mov dword [dap+8],  FAT_LBA_START
    mov dword [dap+12], 0

    mov cx, FAT_SECTORS / 64

.loop:
    mov si, dap
    mov ah, 0x42
    mov dl, [boot_drive]
    int 0x13
    jc .fail

    add word [dap+6],  0x0800
    add dword [dap+8], 64
    loop .loop

    mov dword [DISK_BUF_FLAG], FAT_SECTORS
    clc
    ret

.fail:
    mov dword [DISK_BUF_FLAG], 0
    stc
    ret

;--------------------
; Загрузка ядра
;--------------------
load_kernel:
    mov ah, 0x41
    mov bx, 0x55AA
    mov dl, [boot_drive]
    int 0x13
    jc .chs

    mov cx, 4
    mov word  [dap+2],  64
    mov word  [dap+4],  0x0000
    mov word  [dap+6],  0x1000
    mov dword [dap+8],  9
    mov dword [dap+12], 0

.lba_loop:
    mov si, dap
    mov ah, 0x42
    mov dl, [boot_drive]
    int 0x13
    jc .error
    add word  [dap+6],  0x0800
    add dword [dap+8],  64
    loop .lba_loop
    clc
    ret

.chs:
    mov ax, KERNEL_OFFSET
    mov es, ax
    xor bx, bx
    mov ah, 0x02
    mov al, 63
    mov ch, 0
    mov cl, 6
    mov dh, 0
    mov dl, [boot_drive]
    int 0x13
    jc .error
    clc
    ret

.error:
    stc
    ret

vesa_init:
    ; получение VBE controller info
    mov  ax, 0x4F00
    push ds
    pop  es
    mov  di, vbe_ctrl_info
    int  0x10
    cmp  ax, 0x004F
    jne  .fail

    ; VideoModePtr в vbe_ctrl_info+0x0E (off) / +0x10 (seg)

    movzx eax, word [vbe_ctrl_info + 0x10]   ; segment (zero-extend в 32 бит)
    movzx ebx, word [vbe_ctrl_info + 0x0E]   ; offset  (zero-extend в 32 бит)
    shl  eax, 4                               ; seg * 16
    add  eax, ebx                             ; + offset
    mov  [vi_modelist_ptr], eax               ; сохранить как dword

    mov  si, mode_wishlist

.wish_loop:
    mov  ax, [si]
    test ax, ax
    jz   .fail    ; конец wishlist, ничего не нашлось

    mov  [vi_wish_w],   ax
    mov  ax, [si+2]
    mov  [vi_wish_h],   ax
    mov  ax, [si+4]
    mov  [vi_wish_bpp], ax

    mov  edi, [vi_modelist_ptr]

.mode_loop:
    mov  cx, [di]
    cmp  cx, 0xFFFF
    je   .next_wish

    push si
    push di

    ; Запросить Mode Info для данного номера режима
    mov  ax, 0x4F01
    push ds
    pop  es
    push cx
    mov  di, vbe_mode_info
    int  0x10
    pop  cx
    cmp  ax, 0x004F
    jne  .mode_next

    ; Проверить: режим поддерживается (бит 0) и LFB есть (бит 7)
    mov  ax, [vbe_mode_info]
    test ax, 0x0001
    jz   .mode_next
    test ax, 0x0080
    jz   .mode_next

    ; Проверить ширину
    mov  ax, [vbe_mode_info + 18]
    cmp  ax, [vi_wish_w]
    jne  .mode_next

    ; Проверить высоту
    mov  ax, [vbe_mode_info + 20]
    cmp  ax, [vi_wish_h]
    jne  .mode_next

    ; Проверить bpp >= минимального
    movzx ax, byte [vbe_mode_info + 25]
    cmp  ax, [vi_wish_bpp]
    jb   .mode_next

    ; Тип памяти: 4=Packed pixel, 6=DirectColor
    movzx ax, byte [vbe_mode_info + 27]
    cmp  ax, 4
    je   .mode_ok
    cmp  ax, 6
    jne  .mode_next

.mode_ok:
    pop  di
    pop  si
    mov  ax, [di]
    mov  [vi_found_mode], ax
    jmp  .activate

.mode_next:
    pop  di
    pop  si
    add  di, 2
    jmp  .mode_loop

.next_wish:
    add  si, 6
    jmp  .wish_loop

.activate:
    ; Перечитать mode info найденного режима
    mov  cx, [vi_found_mode]
    mov  ax, 0x4F01
    push ds
    pop  es
    mov  di, vbe_mode_info
    int  0x10
    cmp  ax, 0x004F
    jne  .fail

    ; Установить режим, бит 14 = использовать LFB
    mov  ax, 0x4F02
    mov  bx, [vi_found_mode]
    or   bx, 0x4000
    int  0x10
    cmp  ax, 0x004F
    jne  .fail

    ; Заполнить VideoInfo по VIDEO_INFO_ADDR
    ; lfb_addr (dword)
    mov  eax, [vbe_mode_info + 40]
    mov  dword [VIDEO_INFO_ADDR + 0], eax

    ; width
    mov  ax, [vbe_mode_info + 18]
    mov  word [VIDEO_INFO_ADDR + 4], ax

    ; height
    mov  ax, [vbe_mode_info + 20]
    mov  word [VIDEO_INFO_ADDR + 6], ax


    movzx eax, word [vbe_mode_info + 50]
    test  eax, eax
    jnz   .pitch_ok
    movzx eax, word [vbe_mode_info + 16]
.pitch_ok:
    mov  dword [VIDEO_INFO_ADDR + 8], eax

    ; bpp
    mov  al, [vbe_mode_info + 25]
    mov  byte [VIDEO_INFO_ADDR + 12], al

    ; Цветовые маски
    movzx ax, byte [vbe_mode_info + 54]   ; LinRedMaskSize
    test  al, al
    jnz   .use_lin_masks
    ; VBE 2.0 fallback: banked masks
    mov  al, [vbe_mode_info + 32]   ; RedFieldPosition
    mov  byte [VIDEO_INFO_ADDR + 13], al
    mov  al, [vbe_mode_info + 31]   ; RedMaskSize
    mov  byte [VIDEO_INFO_ADDR + 14], al
    mov  al, [vbe_mode_info + 34]   ; GreenFieldPosition
    mov  byte [VIDEO_INFO_ADDR + 15], al
    mov  al, [vbe_mode_info + 33]   ; GreenMaskSize
    mov  byte [VIDEO_INFO_ADDR + 16], al
    mov  al, [vbe_mode_info + 36]   ; BlueFieldPosition
    mov  byte [VIDEO_INFO_ADDR + 17], al
    mov  al, [vbe_mode_info + 35]   ; BlueMaskSize
    mov  byte [VIDEO_INFO_ADDR + 18], al
    jmp  .masks_store_done
.use_lin_masks:
    ; VBE 3.0: linear masks
    mov  al, [vbe_mode_info + 55]   ; LinRedFieldPosition
    mov  byte [VIDEO_INFO_ADDR + 13], al
    mov  al, [vbe_mode_info + 54]   ; LinRedMaskSize
    mov  byte [VIDEO_INFO_ADDR + 14], al
    mov  al, [vbe_mode_info + 57]   ; LinGreenFieldPosition
    mov  byte [VIDEO_INFO_ADDR + 15], al
    mov  al, [vbe_mode_info + 56]   ; LinGreenMaskSize
    mov  byte [VIDEO_INFO_ADDR + 16], al
    mov  al, [vbe_mode_info + 59]   ; LinBlueFieldPosition
    mov  byte [VIDEO_INFO_ADDR + 17], al
    mov  al, [vbe_mode_info + 58]   ; LinBlueMaskSize
    mov  byte [VIDEO_INFO_ADDR + 18], al
.masks_store_done:

    movzx ax, byte [VIDEO_INFO_ADDR + 12]
    cmp  ax, 8
    jne  .masks_done
    mov  byte [VIDEO_INFO_ADDR + 13], 5   ; red_pos
    mov  byte [VIDEO_INFO_ADDR + 14], 3   ; red_size
    mov  byte [VIDEO_INFO_ADDR + 15], 2   ; green_pos
    mov  byte [VIDEO_INFO_ADDR + 16], 3   ; green_size
    mov  byte [VIDEO_INFO_ADDR + 17], 0   ; blue_pos
    mov  byte [VIDEO_INFO_ADDR + 18], 2   ; blue_size

.masks_done:

    clc
    ret

.fail:
    stc
    ret

;--------------------
; Меню
;--------------------
draw_menu:
    call clear_screen
    mov dh, 6
    mov dl, 22
    mov cl, 36
    mov ch, 14
    call draw_box

    mov dh, 8
    mov dl, 24
    mov si, title_txt
    mov bl, 0x0F
    call print_at_color

    mov dh, 11
    mov dl, 24
    mov si, menu1_txt
    mov bl, 0x07
    cmp byte [selected], 0
    jne .m1
    mov bl, 0x70
.m1:
    call print_at_color

    mov dh, 13
    mov dl, 24
    mov si, menu2_txt
    mov bl, 0x0F
    cmp byte [selected], 1
    jne .m2
    mov bl, 0x70
.m2:
    call print_at_color

    mov dh, 17
    mov dl, 24
    mov si, hint_txt
    mov bl, 0x08
    call print_at_color
    ret

;--------------------
; Рамка
;--------------------
draw_box:
    pusha
    push es
    mov ax, 0xB800
    mov es, ax
    mov [.w], cl
    mov [.h], ch
    mov [.r], dh
    mov [.c], dl
    call .pos
    mov ah, 0x0F
    mov al, 0xC9
    stosw
    mov al, 0xCD
    mov cl, [.w]
    sub cl, 2
    xor ch, ch
.tl: stosw
    loop .tl
    mov al, 0xBB
    stosw
    mov cl, [.h]
    sub cl, 2
    xor ch, ch
.sl:
    inc byte [.r]
    call .pos
    mov ah, 0x0F
    mov al, 0xBA
    stosw
    push cx
    mov cl, [.w]
    sub cl, 2
    xor ch, ch
    mov ax, 0x0720
.fl: stosw
    loop .fl
    pop cx
    mov ah, 0x0F
    mov al, 0xBA
    stosw
    loop .sl
    inc byte [.r]
    call .pos
    mov ah, 0x0F
    mov al, 0xC8
    stosw
    mov al, 0xCD
    mov cl, [.w]
    sub cl, 2
    xor ch, ch
.bl: stosw
    loop .bl
    mov al, 0xBC
    stosw
    pop es
    popa
    ret
.w: db 0
.h: db 0
.r: db 0
.c: db 0
.pos:
    push ax
    push bx
    mov al, [.r]
    mov bl, 80
    mul bl
    mov di, ax
    mov al, [.c]
    xor ah, ah
    add di, ax
    shl di, 1
    pop bx
    pop ax
    ret

clear_screen:
    pusha
    push es
    mov ax, 0xB800
    mov es, ax
    xor di, di
    mov ax, 0x0720
    mov cx, 80*25
    rep stosw
    pop es
    popa
    ret

print_at_color:
    pusha
    push es
    mov ax, 0xB800
    mov es, ax
    mov al, dh
    mov ah, 80
    mul ah
    mov di, ax
    mov al, dl
    xor ah, ah
    add di, ax
    shl di, 1
    mov ah, bl
.lp:
    lodsb
    test al, al
    jz .dn
    stosw
    jmp .lp
.dn:
    pop es
    popa
    ret

print_at:
    mov bl, 0x07
    call print_at_color
    ret

get_key:
    mov ah, 0
    int 0x16
    ret

do_reboot:
    call clear_screen
    mov dh, 12
    mov dl, 30
    mov si, reboot_txt
    call print_at
    mov cx, 0x001E
    mov dx, 0x8480
    mov ah, 0x86
    int 0x15
    mov al, 0xFE
    out 0x64, al
    jmp 0xFFFF:0x0000


;--------------------
; GDT
;--------------------
align 8
gdt_start:
    dq 0
gdt_code:
    dw 0xFFFF, 0
    db 0, 10011010b, 11001111b, 0
gdt_data:
    dw 0xFFFF, 0
    db 0, 10010010b, 11001111b, 0
gdt_user_code:
    dw 0xFFFF, 0
    db 0, 11011010b, 11001111b, 0
gdt_user_data:
    dw 0xFFFF, 0
    db 0, 11010010b, 11001111b, 0
gdt_tss:
    dq 0
gdt_end:

gdt_desc:
    dw gdt_end - gdt_start - 1
    dd gdt_start

CODE_SEG equ gdt_code - gdt_start
DATA_SEG equ gdt_data - gdt_start

enter_pm:
    cli
    mov al, [boot_drive]
    mov [DISK_BUF_DRIVE], al
    in al, 0x92
    or al, 2
    out 0x92, al
    lgdt [gdt_desc]
    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp CODE_SEG:pm_start

[BITS 32]
pm_start:
    mov ax, DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000
    jmp CODE_SEG:0x10000

times 4096-($-$$) db 0