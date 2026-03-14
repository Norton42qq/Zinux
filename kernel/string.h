#ifndef STRING_H
#define STRING_H

#include <stdint.h>
#include <stddef.h>

size_t strlen(const char* str);
int strcmp(const char* s1, const char* s2);
int strncmp(const char* s1, const char* s2, size_t n);
char* strcpy(char* dest, const char* src);
char* strncpy(char* dest, const char* src, size_t n);
char* strcat(char* dest, const char* src);
char* strchr(const char* str, int c);
char* strstr(const char* haystack, const char* needle);

void* memset(void* dest, int c, size_t n);
void* memcpy(void* dest, const void* src, size_t n);
int memcmp(const void* s1, const void* s2, size_t n);
void* memmove(void* dest, const void* src, size_t n);

int atoi(const char* str);
void itoa(int value, char* str, int base);
void utoa(uint32_t value, char* str, int base);

char tolower(char c);
char toupper(char c);
int isdigit(char c);
int isalpha(char c);
int isspace(char c);

char* strtok(char* str, const char* delim);
void str_trim(char* str);

#endif