#ifndef LIBC_ASSERT_H
#define LIBC_ASSERT_H

void _assert_fail(const char* expr, const char* file, int line) __attribute__((noreturn));

#ifdef NDEBUG
#define assert(expr) ((void)0)
#else
#define assert(expr) \
    ((expr) ? (void)0 : _assert_fail(#expr, __FILE__, __LINE__))
#endif

#endif