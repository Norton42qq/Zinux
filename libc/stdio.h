#ifndef LIBC_STDIO_H
#define LIBC_STDIO_H

#include "types.h"

// Максимальная длина пути
#define PATH_MAX 128

// Размер буфера FILE
#define FILE_BUF_SIZE 512

// Режимы позиционирования для fseek
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

// EOF
#define EOF (-1)

// Дескриптор файла
typedef struct {
    char     name[PATH_MAX];  // имя файла на диске
    uint8_t* buf;             // буфер содержимого (выделяется через malloc)
    uint32_t size;            // размер файла
    uint32_t pos;             // текущая позиция чтения/записи
    uint8_t  writable;        // открыт на запись
    uint8_t  dirty;           // буфер изменён, нужна запись на диск
    uint8_t  valid;           // слот занят
} FILE;

#define FOPEN_MAX 8  // максимум одновременно открытых файлов

// Открытие и закрытие
FILE* fopen(const char* path, const char* mode);
int   fclose(FILE* fp);
int   fflush(FILE* fp);

// Чтение
size_t fread(void* buf, size_t size, size_t count, FILE* fp);
int    fgetc(FILE* fp);
char*  fgets(char* buf, int n, FILE* fp);

// Запись
size_t fwrite(const void* buf, size_t size, size_t count, FILE* fp);
int    fputc(int c, FILE* fp);
int    fputs(const char* s, FILE* fp);

// Позиционирование
int    fseek(FILE* fp, int32_t offset, int whence);
int    ftell(FILE* fp);
void   rewind(FILE* fp);
int    feof(FILE* fp);

// Форматированный вывод на экран
int printf(const char* fmt, ...);
int vprintf(const char* fmt, __builtin_va_list ap);

// Форматированный вывод в файл
int fprintf(FILE* fp, const char* fmt, ...);

// Форматированный вывод в строку
int sprintf(char* buf, const char* fmt, ...);
int snprintf(char* buf, size_t n, const char* fmt, ...);
int vsprintf(char* buf, const char* fmt, __builtin_va_list ap);
int vsnprintf(char* buf, size_t n, const char* fmt, __builtin_va_list ap);

// Символьный вывод на экран
int putchar(int c);
int puts(const char* s);

#endif