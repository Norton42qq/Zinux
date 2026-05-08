#include "string.h"
#include "ctype.h"
#include "memory.h"

size_t strlen(const char* s) {
    size_t n = 0;
    while (s[n]) n++;
    return n;
}

int strcmp(const char* s1, const char* s2) {
    while (*s1 && *s1 == *s2) { s1++; s2++; }
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

int strncmp(const char* s1, const char* s2, size_t n) {
    while (n && *s1 && *s1 == *s2) { s1++; s2++; n--; }
    if (n == 0) return 0;
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

int strcasecmp(const char* s1, const char* s2) {
    while (*s1 && tolower(*s1) == tolower(*s2)) { s1++; s2++; }
    return tolower(*(unsigned char*)s1) - tolower(*(unsigned char*)s2);
}

int strncasecmp(const char* s1, const char* s2, size_t n) {
    while (n && *s1 && tolower(*s1) == tolower(*s2)) { s1++; s2++; n--; }
    if (n == 0) return 0;
    return tolower(*(unsigned char*)s1) - tolower(*(unsigned char*)s2);
}

char* strcpy(char* dst, const char* src) {
    char* r = dst;
    while ((*dst++ = *src++));
    return r;
}

char* strncpy(char* dst, const char* src, size_t n) {
    char* r = dst;
    while (n && (*dst++ = *src++)) n--;
    while (n--) *dst++ = '\0';
    return r;
}

char* strcat(char* dst, const char* src) {
    char* r = dst;
    while (*dst) dst++;
    while ((*dst++ = *src++));
    return r;
}

char* strncat(char* dst, const char* src, size_t n) {
    char* r = dst;
    while (*dst) dst++;
    while (n-- && (*dst++ = *src++));
    *dst = '\0';
    return r;
}

char* strchr(const char* s, int c) {
    while (*s) {
        if (*s == (char)c) return (char*)s;
        s++;
    }
    return (c == '\0') ? (char*)s : NULL;
}

char* strrchr(const char* s, int c) {
    const char* last = NULL;
    while (*s) {
        if (*s == (char)c) last = s;
        s++;
    }
    return (c == '\0') ? (char*)s : (char*)last;
}

char* strstr(const char* h, const char* n) {
    size_t nlen = strlen(n);
    if (nlen == 0) return (char*)h;
    while (*h) {
        if (strncmp(h, n, nlen) == 0) return (char*)h;
        h++;
    }
    return NULL;
}

// strtok использует статический указатель — не реентерабелен
static char* _strtok_ptr = NULL;

char* strtok(char* s, const char* delim) {
    return strtok_r(s, delim, &_strtok_ptr);
}

char* strtok_r(char* s, const char* delim, char** saveptr) {
    if (s) *saveptr = s;
    if (!*saveptr) return NULL;

    // пропускаем ведущие разделители
    while (**saveptr && strchr(delim, **saveptr)) (*saveptr)++;

    if (!**saveptr) { *saveptr = NULL; return NULL; }

    char* start = *saveptr;

    while (**saveptr && !strchr(delim, **saveptr)) (*saveptr)++;

    if (**saveptr) *(*saveptr)++ = '\0';
    else           *saveptr = NULL;

    return start;
}

size_t strspn(const char* s, const char* accept) {
    size_t n = 0;
    while (*s && strchr(accept, *s)) { s++; n++; }
    return n;
}

size_t strcspn(const char* s, const char* reject) {
    size_t n = 0;
    while (*s && !strchr(reject, *s)) { s++; n++; }
    return n;
}

char* strpbrk(const char* s, const char* accept) {
    while (*s) {
        if (strchr(accept, *s)) return (char*)s;
        s++;
    }
    return NULL;
}

void* memset(void* dst, int c, size_t n) {
    unsigned char* p = dst;
    while (n--) *p++ = (unsigned char)c;
    return dst;
}

void* memcpy(void* dst, const void* src, size_t n) {
    unsigned char* d = dst;
    const unsigned char* s = src;
    while (n--) *d++ = *s++;
    return dst;
}

void* memmove(void* dst, const void* src, size_t n) {
    unsigned char* d = dst;
    const unsigned char* s = src;
    if (d < s) {
        while (n--) *d++ = *s++;
    } else {
        d += n; s += n;
        while (n--) *--d = *--s;
    }
    return dst;
}

int memcmp(const void* s1, const void* s2, size_t n) {
    const unsigned char* a = s1;
    const unsigned char* b = s2;
    while (n--) {
        if (*a != *b) return *a - *b;
        a++; b++;
    }
    return 0;
}

void* memchr(const void* s, int c, size_t n) {
    const unsigned char* p = s;
    while (n--) {
        if (*p == (unsigned char)c) return (void*)p;
        p++;
    }
    return NULL;
}

char* strdup(const char* s) {
    size_t len = strlen(s) + 1;
    char* copy = malloc(len);
    if (copy) memcpy(copy, s, len);
    return copy;
}

char* strndup(const char* s, size_t n) {
    size_t len = strlen(s);
    if (len > n) len = n;
    char* copy = malloc(len + 1);
    if (copy) {
        memcpy(copy, s, len);
        copy[len] = '\0';
    }
    return copy;
}

char* strtrim(char* s) {
    // убираем хвостовые пробелы
    int end = (int)strlen(s) - 1;
    while (end >= 0 && isspace((unsigned char)s[end])) end--;
    s[end + 1] = '\0';

    // убираем ведущие пробелы
    char* p = s;
    while (*p && isspace((unsigned char)*p)) p++;
    if (p != s) memmove(s, p, strlen(p) + 1);

    return s;
}

void strrev(char* s) {
    char* end = s + strlen(s) - 1;
    while (s < end) {
        char tmp = *s;
        *s++ = *end;
        *end-- = tmp;
    }
}