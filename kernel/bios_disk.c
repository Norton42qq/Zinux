#include "ata.h"
#include "string.h"
#include "io.h"

// Адреса для stage2
#define DISK_BUF_FLAG   (*(volatile uint32_t*)0x8004)
#define DISK_BUF_DRIVE  (*(volatile uint8_t* )0x8008)
#define DISK_BUF_BASE   ((uint8_t*)0x50000)
#define SECTOR_SIZE     512
#define FAT_PART_START  2048
// ----
#define MAX_DIRTY       512
static uint8_t dirty_bitmap[MAX_DIRTY / 8 + 1];

static inline void dirty_set(uint32_t i) {
    if (i < MAX_DIRTY) dirty_bitmap[i / 8] |= (1 << (i % 8));
}
static inline void dirty_clr(uint32_t i) {
    if (i < MAX_DIRTY) dirty_bitmap[i / 8] &= ~(1 << (i % 8));
}
static inline int dirty_get(uint32_t i) {
    if (i >= MAX_DIRTY) return 0;
    return (dirty_bitmap[i / 8] >> (i % 8)) & 1;
}

// Состояние
static uint32_t buf_sectors = 0;
static int      buf_valid   = 0;
static int      ata_hw_ok   = 0;
static uint8_t  boot_drive  = 0x80;

// Прыжок из PM в RM для работы записи данных на накопитель
extern int pm_to_rm_write(uint32_t lba, uint8_t drive, void* buf);

// HW ATA PIO
#define ATA_DATA       0x1F0
#define ATA_STATUS     0x1F7
#define ATA_COMMAND    0x1F7
#define ATA_DRIVE      0x1F6
#define ATA_SECTOR_CNT 0x1F2
#define ATA_LBA_LO     0x1F3
#define ATA_LBA_MID    0x1F4
#define ATA_LBA_HI     0x1F5
#define ATA_SR_BSY     0x80
#define ATA_SR_DRDY    0x40
#define ATA_SR_DRQ     0x08
#define ATA_SR_ERR     0x01
#define ATA_CMD_READ   0x20
#define ATA_CMD_WRITE  0x30
#define ATA_CMD_FLUSH  0xE7

static int hw_wait_ready(void) {
    for (int i = 0; i < 100000; i++) {
        uint8_t s = inb(ATA_STATUS);
        if (s == 0xFF) return -1;
        if (!(s & ATA_SR_BSY) && (s & ATA_SR_DRDY)) return 0;
    }
    return -1;
}

static int hw_wait_drq(void) {
    for (int i = 0; i < 100000; i++) {
        uint8_t s = inb(ATA_STATUS);
        if (s == 0xFF) return -1;
        if (s & ATA_SR_ERR) return -1;
        if (s & ATA_SR_DRQ) return 0;
    }
    return -1;
}

static int hw_read(uint32_t lba, void* buf) {
    uint16_t* b = (uint16_t*)buf;
    if (hw_wait_ready() != 0) return -1;
    outb(ATA_DRIVE,      0xE0 | ((lba >> 24) & 0x0F));
    outb(ATA_SECTOR_CNT, 1);
    outb(ATA_LBA_LO,     (uint8_t)lba);
    outb(ATA_LBA_MID,    (uint8_t)(lba >> 8));
    outb(ATA_LBA_HI,     (uint8_t)(lba >> 16));
    outb(ATA_COMMAND,    ATA_CMD_READ);
    if (hw_wait_drq() != 0) return -1;
    for (int i = 0; i < 256; i++) b[i] = inw(ATA_DATA);
    return 0;
}

static int hw_write(uint32_t lba, const void* buf) {
    const uint16_t* b = (const uint16_t*)buf;
    if (hw_wait_ready() != 0) return -1;
    outb(ATA_DRIVE,      0xE0 | ((lba >> 24) & 0x0F));
    outb(ATA_SECTOR_CNT, 1);
    outb(ATA_LBA_LO,     (uint8_t)lba);
    outb(ATA_LBA_MID,    (uint8_t)(lba >> 8));
    outb(ATA_LBA_HI,     (uint8_t)(lba >> 16));
    outb(ATA_COMMAND,    ATA_CMD_WRITE);
    if (hw_wait_drq() != 0) return -1;
    for (int i = 0; i < 256; i++) outw(ATA_DATA, b[i]);
    outb(ATA_COMMAND, ATA_CMD_FLUSH);
    hw_wait_ready();
    return 0;
}

// Публичный API

void ata_init(void) {
    memset(dirty_bitmap, 0, sizeof(dirty_bitmap));

    buf_sectors = DISK_BUF_FLAG;
    buf_valid   = (buf_sectors > 0);
    boot_drive  = DISK_BUF_DRIVE;

    if (inb(ATA_STATUS) == 0xFF) {
        ata_hw_ok = 0;
        return;
    }
    outb(ATA_DRIVE, 0xA0);
    for (int i = 0; i < 15; i++) inb(ATA_STATUS);
    if (hw_wait_ready() == 0) ata_hw_ok = 1;
}

int ata_read_sector(uint32_t lba, void* buffer) {
    if (buf_valid) {
        if (lba < FAT_PART_START) return -1;
        uint32_t idx = lba - FAT_PART_START;
        if (idx >= buf_sectors) return -1;
        memcpy(buffer, DISK_BUF_BASE + idx * SECTOR_SIZE, SECTOR_SIZE);
        return 0;
    }
    if (ata_hw_ok) return hw_read(lba, buffer);
    return -1;
}

int ata_write_sector(uint32_t lba, const void* buffer) {
    if (buf_valid) {
        if (lba < FAT_PART_START) return -1;
        uint32_t idx = lba - FAT_PART_START;
        if (idx >= buf_sectors) return -1;
        memcpy(DISK_BUF_BASE + idx * SECTOR_SIZE, buffer, SECTOR_SIZE);
        dirty_set(idx);
        return 0;
    }
    if (ata_hw_ok) return hw_write(lba, buffer);
    return -1;
}

int ata_read_sectors(uint32_t lba, uint32_t count, void* buffer) {
    uint8_t* buf = (uint8_t*)buffer;
    for (uint32_t i = 0; i < count; i++)
        if (ata_read_sector(lba + i, buf + i * SECTOR_SIZE) != 0) return -1;
    return 0;
}

int ata_write_sectors(uint32_t lba, uint32_t count, const void* buffer) {
    const uint8_t* buf = (const uint8_t*)buffer;
    for (uint32_t i = 0; i < count; i++)
        if (ata_write_sector(lba + i, buf + i * SECTOR_SIZE) != 0) return -1;
    return 0;
}

/*
  ata_flush - сбрасывает грязные RAM секторы на физический диск.

  Принцип работы:
  1. Если ATA PIO доступен (QEMU/IDE) - пишем напрямую через порты 0x1F0.
      Быстро, без переключений режима.
  2. Если ATA PIO недоступен (USB флешка, виртуальный диск и др.) - используем 
      трамплин PM -> RM -> PM и вызываем INT 13h AH=43h.
      Работает с любым носителем который поддерживает BIOS Int13 Extensions (почти любое современное оборудование).
 
  Возврат: 0 = OK, -N = количество ошибок.
 */
int ata_flush(void) {
    if (!buf_valid) return 0;

    int errors = 0;
    uint32_t limit = (buf_sectors < MAX_DIRTY) ? buf_sectors : MAX_DIRTY;

    for (uint32_t i = 0; i < limit; i++) {
        if (!dirty_get(i)) continue;

        uint32_t lba  = FAT_PART_START + i;
        void*    sect = DISK_BUF_BASE + i * SECTOR_SIZE;
        int ok = -1;

        if (ata_hw_ok) {
            // Путь 1: ATA PIO - прямая запись из PM, без переключений
            ok = hw_write(lba, sect);
        } else {
            // Путь 2: INT 13h через трамплин PM→RM→PM
            // Сектор должен быть <1MB
            ok = pm_to_rm_write(lba, boot_drive, sect);
        }

        if (ok == 0) {
            dirty_clr(i);
        } else {
            errors++;
        }
    }
    return -errors;
}

int ata_dirty_count(void) {
    int count = 0;
    uint32_t limit = (buf_sectors < MAX_DIRTY) ? buf_sectors : MAX_DIRTY;
    for (uint32_t i = 0; i < limit; i++)
        if (dirty_get(i)) count++;
    return count;
}

int ata_identify(void) {
    return (buf_valid || ata_hw_ok) ? 0 : -1;
}