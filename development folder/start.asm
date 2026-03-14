[BITS 32]

global _start
extern main

section .text

_start:
    mov  eax, [esp+8]   ; argv
    mov  ecx, [esp+4]   ; argc
    push eax            ; push argv
    push ecx            ; push argc
    call main
    add  esp, 8
    ret                 ; возврат в ядро