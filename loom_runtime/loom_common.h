#ifndef LOOM_COMMON_H
#define LOOM_COMMON_H

typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef unsigned long long u64;
typedef signed char i8;
typedef signed short i16;
typedef signed int i32;
typedef signed long long i64;

typedef float f32;
typedef double f64;

typedef unsigned long long usize;
typedef char* str;

#ifndef m_namespace
#   define m_namespace(namespace, identifier) namespace ## identifier
#endif

typedef struct str_t {
    char *data;
    usize size;
} str_t;


// TODO: passing into interfaces expecting a cstring
// typedef str_t str;
#endif //LOOM_COMMON_H
