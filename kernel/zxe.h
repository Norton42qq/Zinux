#ifndef ZXE_H
#define ZXE_H

#include <stdint.h>

#define ZXE_MAGIC           0x01455A58
#define ZXE_VERSION         1
#define ZXE_MAX_SIZE        (64 * 1024) // 64 KB
#define ZXE_LOAD_ADDRESS    0x200000 // 2 MB
#define ZXE_OK              0
#define ZXE_ERROR           -1
#define ZXE_NOT_FOUND       -2
#define ZXE_INVALID         -3
#define ZXE_TOO_LARGE       -4

typedef struct {
    uint32_t magic;             // 0x00: Магическое число
    uint8_t  version;           // 0x04: Версия формата
    uint8_t  flags;             // 0x05: Флаги
    uint16_t header_size;       // 0x06: Размер заголовка
    uint32_t code_size;         // 0x08: Размер кода
    uint32_t data_size;         // 0x0C: Размер данных
    uint32_t bss_size;          // 0x10: Размер BSS
    uint32_t entry_offset;      // 0x14: Смещение точки входа
    char     name[16];          // 0x18: Имя программы
    char     reserved[8];       // 0x28: Зарезервировано
} __attribute__((packed)) zxe_header_t;

typedef int (*zxe_entry_fn)(int argc, char* argv[]);

void zxe_init(void);
int zxe_load(const char* filename);
int zxe_run(const char* filename, int argc, char* argv[]);
int zxe_validate(const zxe_header_t* header);
void zxe_get_last_error(char* buffer, int size);

#endif