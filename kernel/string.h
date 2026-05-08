#ifndef STRING_H
#define STRING_H

// kernel/string.h перенаправляет в libc
// Все файлы ядра продолжают писать #include "string.h" как раньше
#include "../libc/string.h"
#include "../libc/ctype.h"
#include "../libc/stdlib.h"

// str_trim — старое имя, теперь это strtrim() в libc
#define str_trim(s) strtrim(s)

#endif