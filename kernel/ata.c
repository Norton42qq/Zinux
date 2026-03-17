#include "ata.h"
#include "io.h"
#include "vesa.h"

// Primary ATA Bus
#define ATA_DATA        0x1F0
#define ATA_ERROR       0x1F1
#define ATA_SECTOR_CNT  0x1F2
#define ATA_LBA_LO      0x1F3
#define ATA_LBA_MID     0x1F4
#define ATA_LBA_HI      0x1F5
#define ATA_DRIVE       0x1F6
#define ATA_STATUS      0x1F7
#define ATA_COMMAND     0x1F7

// Статусы
#define ATA_SR_BSY      0x80    // Busy
#define ATA_SR_DRDY     0x40    // Drive ready
#define ATA_SR_DRQ      0x08    // Data request ready
#define ATA_SR_ERR      0x01    // Error

// Команды
#define ATA_CMD_READ    0x20
#define ATA_CMD_WRITE   0x30
#define ATA_CMD_IDENTIFY 0xEC

static int ata_initialized = 0;

// Ожидание готовности диска
static int ata_wait_ready(void) {
    uint8_t status;
    int timeout = 100000;
    
    while (timeout--) {
        status = inb(ATA_STATUS);
        if (!(status & ATA_SR_BSY) && (status & ATA_SR_DRDY)) {
            return 0;
        }
    }
    return -1;
}

// Ожидание DRQ
static int ata_wait_drq(void) {
    uint8_t status;
    int timeout = 100000;
    
    while (timeout--) {
        status = inb(ATA_STATUS);
        if (status & ATA_SR_ERR) return -1;
        if (status & ATA_SR_DRQ) return 0;
    }
    return -1;
}

// Инициализация
void ata_init(void) {
    // Выбор master
    outb(ATA_DRIVE, 0xA0);
    
    // задержка
    for (int i = 0; i < 15; i++) {
        inb(ATA_STATUS);
    }
    
    if (ata_wait_ready() == 0) {
        ata_initialized = 1;
    }
}

// Чтение одного сектора (512 байт)
int ata_read_sector(uint32_t lba, void* buffer) {
    if (!ata_initialized) return -1;
    
    uint16_t* buf = (uint16_t*)buffer;
    
    // Ожидание готовности
    if (ata_wait_ready() != 0) {
        return -1;
    }
    
    // Настройка LBA28
    outb(ATA_DRIVE, 0xE0 | ((lba >> 24) & 0x0F));
    outb(ATA_SECTOR_CNT, 1);
    outb(ATA_LBA_LO, (uint8_t)lba);
    outb(ATA_LBA_MID, (uint8_t)(lba >> 8));
    outb(ATA_LBA_HI, (uint8_t)(lba >> 16));
    outb(ATA_COMMAND, ATA_CMD_READ);
    
    // Ожидание DRQ
    if (ata_wait_drq() != 0) {
        return -1;
    }
    
    // Чтение данных (256 слов = 512 байт)
    for (int i = 0; i < 256; i++) {
        buf[i] = inw(ATA_DATA);
    }
    
    return 0;
}

// Чтение нескольких секторов
int ata_read_sectors(uint32_t lba, uint32_t count, void* buffer) {
    uint8_t* buf = (uint8_t*)buffer;
    
    for (uint32_t i = 0; i < count; i++) {
        if (ata_read_sector(lba + i, buf + (i * 512)) != 0) {
            return -1;
        }
    }
    
    return 0;
}

// Запись сектора
int ata_write_sector(uint32_t lba, const void* buffer) {
    if (!ata_initialized) return -1;
    
    const uint16_t* buf = (const uint16_t*)buffer;
    
    if (ata_wait_ready() != 0) {
        return -1;
    }
    
    outb(ATA_DRIVE, 0xE0 | ((lba >> 24) & 0x0F));
    outb(ATA_SECTOR_CNT, 1);
    outb(ATA_LBA_LO, (uint8_t)lba);
    outb(ATA_LBA_MID, (uint8_t)(lba >> 8));
    outb(ATA_LBA_HI, (uint8_t)(lba >> 16));
    outb(ATA_COMMAND, ATA_CMD_WRITE);
    
    if (ata_wait_drq() != 0) {
        return -1;
    }
    
    for (int i = 0; i < 256; i++) {
        outw(ATA_DATA, buf[i]);
    }
    
    outb(ATA_COMMAND, 0xE7);
    ata_wait_ready();
    
    return 0;
}

// Запись нескольких секторов
int ata_write_sectors(uint32_t lba, uint32_t count, const void* buffer) {
    const uint8_t* buf = (const uint8_t*)buffer;
    
    for (uint32_t i = 0; i < count; i++) {
        if (ata_write_sector(lba + i, buf + (i * 512)) != 0) {
            return -1;
        }
    }
    
    return 0;
}

// Проверка существования диска
int ata_identify(void) {
    if (!ata_initialized) return -1;
    
    outb(ATA_DRIVE, 0xA0);
    outb(ATA_SECTOR_CNT, 0);
    outb(ATA_LBA_LO, 0);
    outb(ATA_LBA_MID, 0);
    outb(ATA_LBA_HI, 0);
    outb(ATA_COMMAND, ATA_CMD_IDENTIFY);
    
    uint8_t status = inb(ATA_STATUS);
    if (status == 0) return -1;
    
    return ata_wait_ready();
}