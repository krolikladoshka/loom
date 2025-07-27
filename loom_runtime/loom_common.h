#ifndef LOOM_COMMON_H
#define LOOM_COMMON_H

#ifndef __HERE__
#define __HERE__ (__FILE__ ":" #__LINE__)
#endif


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
typedef void* (*coroutine_func_pointer_t)(void*); // why

typedef struct function_pointer_t {
    void (*function)(void**);
    usize args_sizes[];
} dfunction_pointer_t;

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
