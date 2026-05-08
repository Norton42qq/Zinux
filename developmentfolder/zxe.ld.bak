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
        *(.bss)
        *(COMMON)
    }
}