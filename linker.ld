ENTRY(_start)

SECTIONS
{

    /* Ядро загружается по адресу 0x10000 */
    . = 0x10000;
    
    .text : {
        *(.text)
    }
    
    .rodata : {
        *(.rodata)
    }
    
    .data : {
        *(.data)
    }
    
    .bss : {
        *(.bss)
        *(COMMON)
    }
}