/* zcore.ld — линкер-скрипт ядра Zinux (Rust-порт). Сохраняет в точности
 * раскладку прежнего linker.ld:
 *   ядро загружается по 0x10000, куча начинается сразу после .bss
 *   (выравнено до 4KB), верхняя граница HEAP_END = 0x1000000 (16MB).
 *
 * Карта памяти:
 *   0x00000 - 0x0FFFF  стек + BIOS данные
 *   0x10000 - ~0x5FFFF ядро (.text/.rodata/.data/.bss)
 *   0x60000 - 0xFFFFF  куча ядра / ZXE программы
 *   0x100000           API таблица (api.h: API_TABLE_ADDRESS)
 *   0x200000           адрес загрузки ZXE (zxe.h: ZXE_LOAD_ADDRESS)
 *   0x200000 - 0xFFFFFF куча (HEAP_END = 16MB)
 */

ENTRY(_start)

SECTIONS
{
    . = 0x10000;

    .text : ALIGN(4) {
        KEEP(*(.text.zinux_entry))
        *(.text .text.*)
    }

    .rodata : ALIGN(4) {
        *(.rodata .rodata.*)
    }

    .data : ALIGN(4) {
        *(.data .data.*)
    }

    .bss : ALIGN(4) {
        *(.bss .bss.*)
        *(COMMON)
    }

    . = ALIGN(4096);
    _heap_start = .;

    /DISCARD/ : {
        *(.eh_frame .eh_frame_hdr)
        *(.comment)
        *(.note .note.*)
        *(.llvm_addrsig)
    }
}