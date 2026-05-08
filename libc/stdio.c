#include "stdio.h"
#include "string.h"
#include "memory.h"
#include "stdlib.h"
#include "ctype.h"

#include "../kernel/drivers/video.h"

// Вывод символа на экран идёт через video

// Нормализация пути для FAT16:
//   ./foo/bar  -> foo/bar
//   /foo/bar   -> foo/bar   (у нас нет абсолютных путей, убираем ведущий /)
//   foo//bar   -> foo/bar   (двойные слеши)
// Результат пишется в dst, возвращает dst.
static const char* normalize_path(const char* path, char* dst, size_t dstsz) {
    // Убираем ведущие ./ и /
    while ((path[0] == '.' && path[1] == '/') || path[0] == '/')
        path += (path[0] == '.') ? 2 : 1;

    size_t i = 0;
    while (*path && i < dstsz - 1) {
        // Схлопываем двойные слеши
        if (*path == '/' && i > 0 && dst[i-1] == '/') { path++; continue; }
        dst[i++] = *path++;
    }
    dst[i] = '\0';
    return dst;
}

// Таблица открытых файлов
static FILE _files[FOPEN_MAX];

// Доступ к FAT16 из ядра
extern int      fat16_read_file(const char* fn, void* buf, uint32_t max);
extern int      fat16_write_file(const char* fn, const void* buf, uint32_t size);
extern uint32_t fat16_file_size(const char* fn);
extern int      fat16_file_exists(const char* fn);

// ─── vsnprintf ───────────────────────────────────────────────────────

// Вывод числа в буфер; возвращает кол-во записанных байт
static int fmt_uint(char* buf, size_t left, uint32_t val, int base,
                    int upper, int width, int zero_pad) {
    char tmp[32];
    int  len = 0;

    if (val == 0) {
        tmp[len++] = '0';
    } else {
        const char* digits = upper ? "0123456789ABCDEF" : "0123456789abcdef";
        while (val) {
            tmp[len++] = digits[val % base];
            val /= base;
        }
    }

    // Дополнение до width
    int pad = width - len;
    int written = 0;

    while (pad-- > 0 && left > 1) {
        *buf++ = zero_pad ? '0' : ' ';
        written++;
        left--;
    }
    // Переворачиваем tmp
    for (int i = len - 1; i >= 0 && left > 1; i--) {
        *buf++ = tmp[i];
        written++;
        left--;
    }
    return written;
}

int vsnprintf(char* buf, size_t n, const char* fmt, __builtin_va_list ap) {
    if (!buf || n == 0) return 0;

    size_t left    = n;
    int    written = 0;

    while (*fmt && left > 1) {
        if (*fmt != '%') {
            *buf++ = *fmt++;
            written++;
            left--;
            continue;
        }

        fmt++; // пропускаем '%'

        // Флаги
        int zero_pad = 0;
        int left_align = 0;
        if (*fmt == '0') { zero_pad = 1; fmt++; }
        if (*fmt == '-') { left_align = 1; fmt++; (void)left_align; }

        // Ширина поля
        int width = 0;
        while (isdigit(*fmt)) width = width * 10 + (*fmt++ - '0');

        // Модификатор длины
        int is_long = 0;
        if (*fmt == 'l') { is_long = 1; fmt++; }

        char spec = *fmt++;
        const char* s;
        int len;

        switch (spec) {
        case 'd': case 'i': {
            long val = is_long ? __builtin_va_arg(ap, long) : __builtin_va_arg(ap, int);
            if (val < 0) {
                if (left > 1) { *buf++ = '-'; written++; left--; }
                val = -val;
            }
            int w = fmt_uint(buf, left, (uint32_t)val, 10, 0, width, zero_pad);
            buf += w; written += w; left -= w;
            break;
        }
        case 'u': {
            uint32_t val = __builtin_va_arg(ap, uint32_t);
            int w = fmt_uint(buf, left, val, 10, 0, width, zero_pad);
            buf += w; written += w; left -= w;
            break;
        }
        case 'x': {
            uint32_t val = __builtin_va_arg(ap, uint32_t);
            int w = fmt_uint(buf, left, val, 16, 0, width, zero_pad);
            buf += w; written += w; left -= w;
            break;
        }
        case 'X': {
            uint32_t val = __builtin_va_arg(ap, uint32_t);
            int w = fmt_uint(buf, left, val, 16, 1, width, zero_pad);
            buf += w; written += w; left -= w;
            break;
        }
        case 'o': {
            uint32_t val = __builtin_va_arg(ap, uint32_t);
            int w = fmt_uint(buf, left, val, 8, 0, width, zero_pad);
            buf += w; written += w; left -= w;
            break;
        }
        case 'p': {
            // указатель как hex с префиксом 0x
            uint32_t val = (uint32_t)__builtin_va_arg(ap, void*);
            if (left > 2) { *buf++ = '0'; *buf++ = 'x'; written += 2; left -= 2; }
            int w = fmt_uint(buf, left, val, 16, 0, 8, 1);
            buf += w; written += w; left -= w;
            break;
        }
        case 'c': {
            char c = (char)__builtin_va_arg(ap, int);
            *buf++ = c; written++; left--;
            break;
        }
        case 's': {
            s = __builtin_va_arg(ap, const char*);
            if (!s) s = "(null)";
            len = (int)strlen(s);
            // Дополнение пробелами слева если ширина задана
            int pad = width - len;
            while (!left_align && pad-- > 0 && left > 1) {
                *buf++ = ' '; written++; left--;
            }
            while (*s && left > 1) {
                *buf++ = *s++; written++; left--;
            }
            break;
        }
        case '%':
            *buf++ = '%'; written++; left--;
            break;
        // Неизвестный спецификатор — пропускаем
        default:
            break;
        }
    }

    *buf = '\0';
    return written;
}

int vsprintf(char* buf, const char* fmt, __builtin_va_list ap) {
    return vsnprintf(buf, 0xFFFFFF, fmt, ap);
}

int snprintf(char* buf, size_t n, const char* fmt, ...) {
    __builtin_va_list ap;
    __builtin_va_start(ap, fmt);
    int r = vsnprintf(buf, n, fmt, ap);
    __builtin_va_end(ap);
    return r;
}

int sprintf(char* buf, const char* fmt, ...) {
    __builtin_va_list ap;
    __builtin_va_start(ap, fmt);
    int r = vsprintf(buf, fmt, ap);
    __builtin_va_end(ap);
    return r;
}

// ─── printf → экран ─────────────────────────────────────────────────

int vprintf(const char* fmt, __builtin_va_list ap) {
    char buf[512];
    int r = vsnprintf(buf, sizeof(buf), fmt, ap);
    for (int i = 0; i < r; i++) vesa_print_char(buf[i]);
    return r;
}

int printf(const char* fmt, ...) {
    __builtin_va_list ap;
    __builtin_va_start(ap, fmt);
    int r = vprintf(fmt, ap);
    __builtin_va_end(ap);
    return r;
}

int putchar(int c) {
    vesa_print_char((char)c);
    return c;
}

int puts(const char* s) {
    while (*s) vesa_print_char(*s++);
    vesa_print_char('\n');
    return 0;
}

// ─── FILE ───────────────────────────────────────────────────────────

static FILE* find_free_slot(void) {
    for (int i = 0; i < FOPEN_MAX; i++) {
        if (!_files[i].valid) return &_files[i];
    }
    return NULL;
}

FILE* fopen(const char* path, const char* mode) {
    if (!path || !mode) return NULL;

    FILE* fp = find_free_slot();
    if (!fp) return NULL;

    // Нормализуем путь: убираем ./ и ведущий /
    char norm[PATH_MAX];
    path = normalize_path(path, norm, sizeof(norm));

    memset(fp, 0, sizeof(FILE));
    strncpy(fp->name, path, PATH_MAX - 1);

    int write_mode = (mode[0] == 'w' || mode[0] == 'a');
    fp->writable = write_mode;

    if (write_mode && mode[0] != 'a') {
        // Режим "w": пустой буфер, файл будет создан при fclose/fflush
        fp->buf  = malloc(FILE_BUF_SIZE);
        fp->size = 0;
        fp->pos  = 0;
        fp->dirty = 1;
        fp->valid = 1;
        return fp;
    }

    // Режим "r" или "a": читаем файл в буфер
    if (!fat16_file_exists(path)) {
        if (mode[0] == 'a') {
            // append: создаём пустой
            fp->buf  = malloc(FILE_BUF_SIZE);
            fp->size = 0;
            fp->pos  = 0;
            fp->dirty = 1;
            fp->valid = 1;
            return fp;
        }
        return NULL; // файл не найден
    }

    uint32_t sz = fat16_file_size(path);
    fp->buf = malloc(sz + 1);
    if (!fp->buf) return NULL;

    if (fat16_read_file(path, fp->buf, sz) < 0) {
        free(fp->buf);
        return NULL;
    }
    fp->size = sz;
    fp->pos  = (mode[0] == 'a') ? sz : 0;
    fp->valid = 1;
    return fp;
}

int fclose(FILE* fp) {
    if (!fp || !fp->valid) return EOF;
    if (fp->dirty && fp->writable) fflush(fp);
    free(fp->buf);
    fp->valid = 0;
    return 0;
}

int fflush(FILE* fp) {
    if (!fp || !fp->dirty) return 0;
    int r = fat16_write_file(fp->name, fp->buf, fp->size);
    fp->dirty = 0;
    return r < 0 ? EOF : 0;
}

size_t fread(void* buf, size_t size, size_t count, FILE* fp) {
    if (!fp || !fp->valid || !buf) return 0;
    size_t bytes = size * count;
    size_t avail = fp->size - fp->pos;
    if (bytes > avail) bytes = avail;
    memcpy(buf, fp->buf + fp->pos, bytes);
    fp->pos += bytes;
    return bytes / size;
}

int fgetc(FILE* fp) {
    if (!fp || !fp->valid || fp->pos >= fp->size) return EOF;
    return (unsigned char)fp->buf[fp->pos++];
}

char* fgets(char* buf, int n, FILE* fp) {
    if (!fp || !fp->valid || n <= 0) return NULL;
    int i = 0;
    while (i < n - 1) {
        int c = fgetc(fp);
        if (c == EOF) { if (i == 0) return NULL; break; }
        buf[i++] = (char)c;
        if (c == '\n') break;
    }
    buf[i] = '\0';
    return buf;
}

size_t fwrite(const void* buf, size_t size, size_t count, FILE* fp) {
    if (!fp || !fp->valid || !fp->writable || !buf) return 0;
    size_t bytes = size * count;
    size_t needed = fp->pos + bytes;

    // Увеличиваем буфер если нужно
    if (needed > fp->size) {
        uint8_t* new_buf = realloc(fp->buf, needed + FILE_BUF_SIZE);
        if (!new_buf) return 0;
        fp->buf  = new_buf;
        fp->size = needed;
    }

    memcpy(fp->buf + fp->pos, buf, bytes);
    fp->pos  += bytes;
    fp->dirty = 1;
    return count;
}

int fputc(int c, FILE* fp) {
    unsigned char ch = (unsigned char)c;
    return fwrite(&ch, 1, 1, fp) == 1 ? c : EOF;
}

int fputs(const char* s, FILE* fp) {
    size_t len = strlen(s);
    return fwrite(s, 1, len, fp) == len ? 0 : EOF;
}

int fseek(FILE* fp, int32_t offset, int whence) {
    if (!fp || !fp->valid) return -1;
    int32_t new_pos;
    if      (whence == SEEK_SET) new_pos = offset;
    else if (whence == SEEK_CUR) new_pos = (int32_t)fp->pos + offset;
    else if (whence == SEEK_END) new_pos = (int32_t)fp->size + offset;
    else return -1;

    if (new_pos < 0) return -1;
    fp->pos = (uint32_t)new_pos;
    return 0;
}

int ftell(FILE* fp) {
    if (!fp || !fp->valid) return EOF;
    return (int)fp->pos;
}

void rewind(FILE* fp) {
    if (fp && fp->valid) fp->pos = 0;
}

int feof(FILE* fp) {
    if (!fp || !fp->valid) return 1;
    return fp->pos >= fp->size;
}

int fprintf(FILE* fp, const char* fmt, ...) {
    char buf[512];
    __builtin_va_list ap;
    __builtin_va_start(ap, fmt);
    int r = vsnprintf(buf, sizeof(buf), fmt, ap);
    __builtin_va_end(ap);
    fwrite(buf, 1, r, fp);
    return r;
}