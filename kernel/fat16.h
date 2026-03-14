#ifndef FAT16_H
#define FAT16_H

#include <stdint.h>

typedef struct {
    char     name[8];
    char     ext[3];
    uint8_t  attributes;
    uint8_t  reserved;
    uint8_t  create_time_ms;
    uint16_t create_time;
    uint16_t create_date;
    uint16_t access_date;
    uint16_t cluster_high;
    uint16_t modify_time;
    uint16_t modify_date;
    uint16_t cluster_low;
    uint32_t file_size;
} __attribute__((packed)) fat16_dir_entry_t;

int      fat16_init       (uint32_t partition_start);
int      fat16_find_file  (const char* filename, fat16_dir_entry_t* entry);
int      fat16_read_file  (const char* filename, void* buffer, uint32_t max_size);
int      fat16_write_file (const char* filename, const void* data, uint32_t size);
int      fat16_mkdir      (const char* dirname);
int      fat16_list_root  (void);
int      fat16_list_dir   (const char* path);
uint32_t fat16_file_size  (const char* filename);
int      fat16_file_exists(const char* filename);

#endif