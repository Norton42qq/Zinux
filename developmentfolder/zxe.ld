/* zxe.ld - Скрипт линковщика для ZXE программ */

ENTRY(_start)

SECTIONS
{
    /* Загрузочный адрес + размер заголовка (48 байт = 0x30) */
    . = 0x200030;
    
    .text : {
        *(.text)
    }
    
    .rodata : {
        *(.rodata*)
    }
    
    .data : {
        *(.data)
    }
    
    .bss : {
        _bss_start = .;
        *(.bss)
        *(COMMON)
        _bss_end = .;
    }

    /* Начало кучи — сразу после конца программы, выровнено до 4KB */
    . = ALIGN(4096);
    _heap_start = .;
}