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

    /* Начало кучи — сразу после конца ядра, выровнено до 4KB.
     * Верхняя граница кучи задана в libc/memory.h как HEAP_END.
     * Сейчас HEAP_END = 0x1000000 (16MB) — нужно для Doom.
     *
     * Карта памяти:
     *   0x00000 - 0x0FFFF  стек + BIOS данные
     *   0x10000 - ~0x5FFFF ядро (.text/.data/.bss)
     *   0x60000 - 0xFFFFF  куча ядра / ZXE программы
     *   0x100000           API таблица (api.h: API_TABLE_ADDRESS)
     *   0x200000           адрес загрузки ZXE (zxe.h: ZXE_LOAD_ADDRESS)
     *   0x200000 - 0xFFFFFF куча (HEAP_END = 16MB)
     *
     * Важно: malloc не трогает 0x100000 и 0x200000 — они лежат
     * внутри кучи. API таблица и ZXE пишутся напрямую по адресам,
     * минуя аллокатор. Так и было до этого, не меняем.
     */
    . = ALIGN(4096);
    _heap_start = .;
}
