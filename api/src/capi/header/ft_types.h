#ifndef FT_TYPES_H
#define FT_TYPES_H

#include <stdint.h>
#include <float.h>
#include <uchar.h>
#include <stdbool.h>

typedef int8_t i8;
typedef int16_t i16;
typedef int32_t i32;
typedef int64_t i64;
typedef __int128_t i128;

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;
typedef __uint128_t u128;

typedef size_t usize;
typedef __ssize_t isize;

typedef unsigned char c8;
typedef char16_t c16;
typedef char32_t c32;

#if (__SIZEOF_FLOAT__ == 4)
typedef float f32;
#endif
#if (__SIZEOF_DOUBLE__ == 8)
typedef double f64;
#endif
#if (__SIZEOF_LONG_DOUBLE__ == 16)
typedef long double f128;
#define FLOAT_128
#elif (__SIZEOF_LONG_DOUBLE__ == 10)
typedef long double f80;
#define FLOAT_80
#endif

#ifdef FLOAT_128
typedef f128 fmax;
#elif defined FLOAT_80
typedef f80 fmax;
#endif

#define OUT
#define NON_NULL

#endif