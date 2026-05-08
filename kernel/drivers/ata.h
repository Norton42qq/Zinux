#ifndef ATA_H
#define ATA_H

#include <stdint.h>

void ata_init(void);

int ata_read_sector(uint32_t lba, void* buffer);
int ata_read_sectors(uint32_t lba, uint32_t count, void* buffer);

int ata_write_sector(uint32_t lba, const void* buffer);
int ata_write_sectors(uint32_t lba, uint32_t count, const void* buffer);


int ata_flush(void); // Возврат: 0 = успех, -N = кол-во ошибок
int ata_dirty_count(void);

int ata_identify(void);

#endif