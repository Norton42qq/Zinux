#include "zxe.h"
#include "fat16.h"
#include "string.h"
#include "vga.h"
#include "api.h"

// Буфер для загрузки программы
static uint8_t* program_space = (uint8_t*)ZXE_LOAD_ADDRESS;
static char last_error[64];

// Инициализация
void zxe_init(void) {
    last_error[0] = '\0';
}

// Валидация заголовка
int zxe_validate(const zxe_header_t* header) {
    if (header->magic != ZXE_MAGIC) {
        strcpy(last_error, "Invalid magic number");
        return ZXE_INVALID;
    }
    
    if (header->version != ZXE_VERSION) {
        strcpy(last_error, "Unsupported version");
        return ZXE_INVALID;
    }
    
    if (header->code_size == 0) {
        strcpy(last_error, "Empty code section");
        return ZXE_INVALID;
    }
    
    if (header->code_size + header->data_size > ZXE_MAX_SIZE) {
        strcpy(last_error, "Program too large");
        return ZXE_TOO_LARGE;
    }
    
    return ZXE_OK;
}

// Запуск программы
int zxe_run(const char* filename, int argc, char* argv[]) {
    char fname[64];
    strncpy(fname, filename, 60);
    
    // возможность запуска без дописания ".zxe"
    int len = strlen(fname);
    if (len < 4 || (strcmp(&fname[len-4], ".zxe") != 0 && 
                    strcmp(&fname[len-4], ".ZXE") != 0)) {
        strcat(fname, ".ZXE");
    }
    
    if (!fat16_file_exists(fname)) {
        strcpy(last_error, "File not found");
        return ZXE_NOT_FOUND;
    }
    
    int bytes = fat16_read_file(fname, program_space, ZXE_MAX_SIZE);
    if (bytes < 0) {
        strcpy(last_error, "Read error");
        return ZXE_ERROR;
    }
    
    zxe_header_t* header = (zxe_header_t*)program_space;
    int result = zxe_validate(header);
    if (result != ZXE_OK) {
        return result;
    }
    
    // Инициализация API (заполняем таблицу функций по 0x100000)
    api_init();
    uint8_t* entry_point = program_space + header->header_size + header->entry_offset;
    
    // Вызов
    zxe_entry_fn entry_fn = (zxe_entry_fn)entry_point;
    int exit_code = entry_fn(argc, argv);
    
    return exit_code;
}

// Получение последней ошибки
void zxe_get_last_error(char* buffer, int size) {
    strncpy(buffer, last_error, size - 1);
    buffer[size - 1] = '\0';
}