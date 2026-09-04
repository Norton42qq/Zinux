//! Перенос `kernel/zxe.c` + `kernel/zxe.h` — запуск .ZXE-программ.
//!
//! Программа грузится во 2 МБ (ZXE_LOAD_ADDRESS), валидируется и выполняется
//! в кольце 2 через `arch::entry::run_user_program`.

use core::ptr;

use zlibk::str::{strcat, strcmp, strcpy, strlen, strncpy};

use crate::api::api_setup_table;
use crate::arch::cpu::run_user_program;
use crate::fat16::{fat16_file_exists, fat16_read_file};

pub const ZXE_MAGIC: u32 = 0x0145_5A58;
pub const ZXE_VERSION: u32 = 1;
pub const ZXE_MAX_SIZE: u32 = 2048 * 1024; // 2 МБ
pub const ZXE_LOAD_ADDRESS: usize = 0x200000;
pub const ZXE_USER_STACK_TOP: u32 = 0x2F0000;

pub const ZXE_OK: i32 = 0;
pub const ZXE_ERROR: i32 = -1;
pub const ZXE_NOT_FOUND: i32 = -2;
pub const ZXE_INVALID: i32 = -3;
pub const ZXE_TOO_LARGE: i32 = -4;

/// Заголовок ZXE (`zxe_header_t`, 48 байт).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct ZxeHeader {
    pub magic: u32,        // 0x00
    pub version: u8,       // 0x04
    pub flags: u8,         // 0x05
    pub header_size: u16,  // 0x06
    pub code_size: u32,    // 0x08
    pub data_size: u32,    // 0x0C
    pub bss_size: u32,     // 0x10
    pub entry_offset: u32, // 0x14
    pub name: [u8; 16],    // 0x18
    pub reserved: [u8; 8], // 0x28
}

static mut LAST_ERROR: [u8; 64] = [0; 64];

/// Инициализация (`zxe_init`).
pub fn zxe_init() {
    unsafe {
        LAST_ERROR[0] = 0;
    }
}

/// Валидация заголовка (`zxe_validate`).
pub fn zxe_validate(header: *const ZxeHeader) -> i32 {
    unsafe {
        if (*header).magic != ZXE_MAGIC {
            strcpy(LAST_ERROR.as_mut_ptr(), b"Invalid magic number\0".as_ptr());
            return ZXE_INVALID;
        }
        if (*header).version as u32 != ZXE_VERSION {
            strcpy(LAST_ERROR.as_mut_ptr(), b"Unsupported version\0".as_ptr());
            return ZXE_INVALID;
        }
        if (*header).code_size == 0 {
            strcpy(LAST_ERROR.as_mut_ptr(), b"Empty code section\0".as_ptr());
            return ZXE_INVALID;
        }
        if (*header).code_size + (*header).data_size > ZXE_MAX_SIZE {
            strcpy(LAST_ERROR.as_mut_ptr(), b"Program too large\0".as_ptr());
            return ZXE_TOO_LARGE;
        }
    }
    ZXE_OK
}

/// Запуск программы (`zxe_run`).
pub fn zxe_run(filename: *const u8, argc: i32, argv: *mut *mut u8) -> i32 {
    // Буфер для загрузки программы (фиксированный адрес 2 МБ).
    let program_space = ZXE_LOAD_ADDRESS as *mut u8;

    let mut fname = [0u8; 64];
    unsafe {
        strncpy(fname.as_mut_ptr(), filename, 60);

        // Возможен запуск без дописывания ".zxe"
        let len = strlen(fname.as_ptr());
        if len < 4
            || (strcmp(fname.as_ptr().add(len - 4), b".zxe\0".as_ptr()) != 0
                && strcmp(fname.as_ptr().add(len - 4), b".ZXE\0".as_ptr()) != 0)
        {
            strcat(fname.as_mut_ptr(), b".ZXE\0".as_ptr());
        }
    }

    let fname_ptr = fname.as_ptr();
    let exists = fat16_file_exists(fname_ptr);
    if exists == 0 {
        unsafe {
            strcpy(LAST_ERROR.as_mut_ptr(), b"File not found\0".as_ptr());
        }
        return ZXE_NOT_FOUND;
    }

    let bytes = fat16_read_file(fname_ptr, program_space, ZXE_MAX_SIZE);
    if bytes < 0 {
        unsafe {
            strcpy(LAST_ERROR.as_mut_ptr(), b"Read error\0".as_ptr());
        }
        return ZXE_ERROR;
    }

    let header = program_space as *const ZxeHeader;
    let result = zxe_validate(header);
    if result != ZXE_OK {
        return result;
    }

    api_setup_table();

    unsafe {
        let entry_offset = (*header).header_size as usize + (*header).entry_offset as usize;
        let entry_point = program_space.add(entry_offset) as u32;
        let exit_code = run_user_program(entry_point, ZXE_USER_STACK_TOP, argc, argv);
        exit_code
    }
}

/// Получение последней ошибки (`zxe_get_last_error`).
pub fn zxe_get_last_error(buffer: *mut u8, size: i32) {
    unsafe {
        strncpy(buffer, LAST_ERROR.as_ptr(), (size - 1) as usize);
        *buffer.add((size - 1) as usize) = 0;
    }
}

/// Получение указателя на буфер ошибки (для отладки).
pub fn zxe_last_error() -> *const u8 {
    unsafe { LAST_ERROR.as_ptr() as *const u8 }
}