#include "assert.h"
#include "stdio.h"

void _assert_fail(const char* expr, const char* file, int line) {
    printf("ASSERT FAILED: %s\n  file: %s\n  line: %d\n", expr, file, line);
    // Зависаем — в ядре abort() вызывает halt
    extern void abort(void) __attribute__((noreturn));
    abort();
}