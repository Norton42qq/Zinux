#include "string.h"

// Длина строки
size_t strlen(const char* str) {
    size_t len = 0;
    while (str[len]) {
        len++;
    }
    return len;
}

// Сравнение строк
int strcmp(const char* s1, const char* s2) {
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

// Сравнение N символов
int strncmp(const char* s1, const char* s2, size_t n) {
    while (n && *s1 && (*s1 == *s2)) {
        s1++;
        s2++;
        n--;
    }
    if (n == 0) return 0;
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

// Копирование строки
char* strcpy(char* dest, const char* src) {
    char* ret = dest;
    while ((*dest++ = *src++));
    return ret;
}

// Копирование N символов
char* strncpy(char* dest, const char* src, size_t n) {
    char* ret = dest;
    while (n && (*dest++ = *src++)) {
        n--;
    }
    while (n--) {
        *dest++ = '\0';
    }
    return ret;
}

// Конкатенация строк
char* strcat(char* dest, const char* src) {
    char* ret = dest;
    while (*dest) {
        dest++;
    }
    while ((*dest++ = *src++));
    return ret;
}

// Поиск символа в строке
char* strchr(const char* str, int c) {
    while (*str) {
        if (*str == (char)c) {
            return (char*)str;
        }
        str++;
    }
    return (c == '\0') ? (char*)str : NULL;
}

// Поиск подстроки
char* strstr(const char* haystack, const char* needle) {
    size_t needle_len = strlen(needle);
    if (needle_len == 0) return (char*)haystack;
    
    while (*haystack) {
        if (strncmp(haystack, needle, needle_len) == 0) {
            return (char*)haystack;
        }
        haystack++;
    }
    return NULL;
}

// Заполнение памяти
void* memset(void* dest, int c, size_t n) {
    unsigned char* p = dest;
    while (n--) {
        *p++ = (unsigned char)c;
    }
    return dest;
}

// Копирование памяти
void* memcpy(void* dest, const void* src, size_t n) {
    unsigned char* d = dest;
    const unsigned char* s = src;
    while (n--) {
        *d++ = *s++;
    }
    return dest;
}

// Сравнение памяти
int memcmp(const void* s1, const void* s2, size_t n) {
    const unsigned char* p1 = s1;
    const unsigned char* p2 = s2;
    while (n--) {
        if (*p1 != *p2) {
            return *p1 - *p2;
        }
        p1++;
        p2++;
    }
    return 0;
}

// Перемещение памяти (с перекрытием)
void* memmove(void* dest, const void* src, size_t n) {
    unsigned char* d = dest;
    const unsigned char* s = src;
    
    if (d < s) {
        while (n--) {
            *d++ = *s++;
        }
    } else {
        d += n;
        s += n;
        while (n--) {
            *--d = *--s;
        }
    }
    return dest;
}

// Преобразование строки в число
int atoi(const char* str) {
    int result = 0;
    int sign = 1;
    
    while (isspace(*str)) str++;
    
    if (*str == '-') {
        sign = -1;
        str++;
    } else if (*str == '+') {
        str++;
    }
    
    while (isdigit(*str)) {
        result = result * 10 + (*str - '0');
        str++;
    }
    
    return sign * result;
}

// Преобразование числа в строку
void itoa(int value, char* str, int base) {
    char* ptr = str;
    char* ptr1 = str;
    char tmp_char;
    int tmp_value;
    
    if (value < 0 && base == 10) {
        *ptr++ = '-';
        ptr1++;
        value = -value;
    }
    
    do {
        tmp_value = value;
        value /= base;
        *ptr++ = "0123456789abcdef"[tmp_value - value * base];
    } while (value);
    
    *ptr-- = '\0';
    
    while (ptr1 < ptr) {
        tmp_char = *ptr;
        *ptr-- = *ptr1;
        *ptr1++ = tmp_char;
    }
}

// Преобразование беззнакового числа в строку
void utoa(uint32_t value, char* str, int base) {
    char* ptr = str;
    char* ptr1 = str;
    char tmp_char;
    uint32_t tmp_value;
    
    do {
        tmp_value = value;
        value /= base;
        *ptr++ = "0123456789abcdef"[tmp_value - value * base];
    } while (value);
    
    *ptr-- = '\0';
    
    while (ptr1 < ptr) {
        tmp_char = *ptr;
        *ptr-- = *ptr1;
        *ptr1++ = tmp_char;
    }
}

// Преобразование в нижний регистр
char tolower(char c) {
    if (c >= 'A' && c <= 'Z') {
        return c + 32;
    }
    return c;
}

// Преобразование в верхний регистр
char toupper(char c) {
    if (c >= 'a' && c <= 'z') {
        return c - 32;
    }
    return c;
}

// Проверка: цифра
int isdigit(char c) {
    return c >= '0' && c <= '9';
}

// Проверка: буква
int isalpha(char c) {
    return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z');
}

// Проверка: пробел
int isspace(char c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' || c == '\f';
}

// Разбор строки по разделителям
static char* strtok_ptr = NULL;

char* strtok(char* str, const char* delim) {
    char* token_start;
    
    if (str != NULL) {
        strtok_ptr = str;
    }
    
    if (strtok_ptr == NULL) {
        return NULL;
    }
    
    // Пропускаем начальные разделители
    while (*strtok_ptr && strchr(delim, *strtok_ptr)) {
        strtok_ptr++;
    }
    
    if (*strtok_ptr == '\0') {
        strtok_ptr = NULL;
        return NULL;
    }
    
    token_start = strtok_ptr;
    
    // Ищем конец токена
    while (*strtok_ptr && !strchr(delim, *strtok_ptr)) {
        strtok_ptr++;
    }
    
    if (*strtok_ptr) {
        *strtok_ptr++ = '\0';
    } else {
        strtok_ptr = NULL;
    }
    
    return token_start;
}

// Удаление пробелов в начале и конце строки
void str_trim(char* str) {
    char* start = str;
    char* end;
    
    // Пропускаем пробелы в начале
    while (isspace(*start)) {
        start++;
    }
    
    if (*start == '\0') {
        str[0] = '\0';
        return;
    }
    
    // Находим конец строки
    end = start + strlen(start) - 1;
    
    // Удаляем пробелы в конце
    while (end > start && isspace(*end)) {
        end--;
    }
    
    *(end + 1) = '\0';
    
    // Перемещаем строку в начало
    if (start != str) {
        memmove(str, start, end - start + 2);
    }
}