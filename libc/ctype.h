#ifndef LIBC_CTYPE_H
#define LIBC_CTYPE_H

#include "types.h"

static inline int isdigit(int c)  { return c >= '0' && c <= '9'; }
static inline int isalpha(int c)  { return (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z'); }
static inline int isalnum(int c)  { return isdigit(c) || isalpha(c); }
static inline int isspace(int c)  { return c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\v' || c == '\f'; }
static inline int isupper(int c)  { return c >= 'A' && c <= 'Z'; }
static inline int islower(int c)  { return c >= 'a' && c <= 'z'; }
static inline int isprint(int c)  { return c >= 0x20 && c <= 0x7E; }
static inline int iscntrl(int c)  { return (c >= 0x00 && c <= 0x1F) || c == 0x7F; }
static inline int isxdigit(int c) { return isdigit(c) || (c >= 'a' && c <= 'f') || (c >= 'A' && c <= 'F'); }
static inline int ispunct(int c)  { return isprint(c) && !isalnum(c) && c != ' '; }

static inline int toupper(int c)  { return islower(c) ? c - 32 : c; }
static inline int tolower(int c)  { return isupper(c) ? c + 32 : c; }

#endif