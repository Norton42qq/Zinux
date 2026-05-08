#ifndef LIBC_STRING_H
#define LIBC_STRING_H

#include "types.h"

// Длина и сравнение
size_t strlen(const char* s);
int    strcmp(const char* s1, const char* s2);
int    strncmp(const char* s1, const char* s2, size_t n);
int    strcasecmp(const char* s1, const char* s2);
int    strncasecmp(const char* s1, const char* s2, size_t n);

// Копирование и конкатенация
char*  strcpy(char* dst, const char* src);
char*  strncpy(char* dst, const char* src, size_t n);
char*  strcat(char* dst, const char* src);
char*  strncat(char* dst, const char* src, size_t n);

// Поиск
char*  strchr(const char* s, int c);
char*  strrchr(const char* s, int c);
char*  strstr(const char* haystack, const char* needle);
char*  strtok(char* s, const char* delim);
char*  strtok_r(char* s, const char* delim, char** saveptr);
size_t strspn(const char* s, const char* accept);
size_t strcspn(const char* s, const char* reject);
char*  strpbrk(const char* s, const char* accept);

// Работа с памятью
void*  memset(void* dst, int c, size_t n);
void*  memcpy(void* dst, const void* src, size_t n);
void*  memmove(void* dst, const void* src, size_t n);
int    memcmp(const void* s1, const void* s2, size_t n);
void*  memchr(const void* s, int c, size_t n);

// Дубликат строки (использует malloc)
char*  strdup(const char* s);
char*  strndup(const char* s, size_t n);

// Преобразования
char*  strtrim(char* s);        // убирает пробелы по краям (in-place)
void   strrev(char* s);         // переворачивает строку (in-place)

#endif