[BITS 32]

global _start
extern main

; Символы из zxe.ld
extern _bss_start
extern _bss_end

section .text

_start:
    ; Обнуляем BSS — без этого глобальные переменные содержат мусор
    mov edi, _bss_start
    mov ecx, _bss_end
    sub ecx, edi
    xor eax, eax
    rep stosb

    mov  eax, [esp+8]   ; argv
    mov  ecx, [esp+4]   ; argc
    push eax
    push ecx
    call main
    add  esp, 8

    ; Завершение через syscall exit (EAX=2, EBX=exit_code)
    mov ebx, eax
    mov eax, 2
    int 0x80
.hang:
    jmp .hang