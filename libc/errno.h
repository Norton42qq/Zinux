#ifndef LIBC_ERRNO_H
#define LIBC_ERRNO_H

#include "types.h"

// Глобальная переменная ошибки
// Функции устанавливают errno при ошибке и возвращают -1 или NULL
extern int errno;

#define EPERM         1
#define ENOENT        2
#define EIO           5
#define EBADF         9
#define ENOMEM       12
#define EACCES       13
#define EFAULT       14
#define EBUSY        16
#define EEXIST       17
#define ENODEV       19
#define ENOTDIR      20
#define EISDIR       21
#define EINVAL       22
#define ENFILE       23
#define EMFILE       24
#define EFBIG        27
#define ENOSPC       28
#define ESPIPE       29
#define EROFS        30
#define EPIPE        32
#define ERANGE       34
#define ENAMETOOLONG 36
#define ENOSYS       38
#define ENOTEMPTY    39
#define EOVERFLOW    75

#define SET_ERRNO(e) do { errno = (e); } while(0)

#endif