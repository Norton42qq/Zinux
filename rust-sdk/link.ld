/*
 * link.ld — линкер-скрипт для ZXE-программ на Rust (Zinux, Ring 2).
 *
 * Ядро загружает файл .zxe по адресу 0x200000 (ZXE_LOAD_ADDRESS), затем
 * исполняет код по адресу base + header_size + entry_offset.
 * header_size = 48 (0x30), поэтому код линкуется с 0x200030.
 *
 * Вся память программы (код + данные + стек от 0x2F0000 вниз) должна
 * лежать в диапазоне 0x200000..0x300000 (проверки user_ptr_ok/user_str_ok).
 * Код не должен превышать 64 KB.
 */

ENTRY(_start)

SECTIONS
{
    . = 0x200030;

    .text : {
        KEEP(*(.text.zinux_entry))
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        _bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        _bss_end = .;
        . = _bss_start + 4;
    }

    /DISCARD/ : {
        *(.eh_frame .eh_frame_hdr)
        *(.comment)
        *(.note .note.*)
        *(.llvm_addrsig)
    }
}