#ifndef LOOM_MEMORY_H
#define LOOM_MEMORY_H

#include <stdlib.h>

// for arenas
#ifndef m_kilobytes
#define m_kilobytes(amount) ((usize) (((usize) amount) * 1024))
#endif


#ifndef m_megabytes
#define m_megabytes(amount) ((usize) ( (m_kilobytes(1024)) * 1024) )
#endif


static inline void* def_malloc(usize size) {
    return malloc(size);
}

#ifndef def_malloc_u8
#define def_malloc_u8(size) ((u8*) def_malloc((size)))
#endif


static inline void def_free(void* memory) {
    free(memory);
}


static inline u8* sigblock_malloc(usize size) {
    // TODO: block signals & context switch
    sigblo
    u8* memory = def_malloc(size);

    return memory;
}

inline void sigblock_free(void* memory) {
    // TODO: block signals & context switch
    def_free(memory);
}

static inline u8* sigblock_realloc(void* memory, usize new_size) {
    return (u8*) realloc(memory, new_size);
}

static inline void memcopy(u8* to, const u8* buffer, usize buffer_size) {
    for (u8* p = to; p < (buffer + buffer_size); ++p, ++buffer) {
        *p = *buffer;
    }
}

typedef struct dynarray_t {
    u8 *data;
    usize size;
    usize capacity;
} dynarray_t;


inline dynarray_t dynarray_new() {
    dynarray_t dynarray = {0};

    return dynarray;
}

inline dynarray_t dynarray_with_capacity(usize capacity) {
    dynarray_t dynarray = {0};

    dynarray.data = sigblock_malloc(capacity);
    dynarray.size = 0;
    dynarray.capacity = capacity;

    return dynarray;
}

inline void dynarray_free(dynarray_t* dynarray) {
    if (dynarray == 0) {
        return;
    }

    sigblock_free(dynarray->data);
}


void dynarray_extend_capacity(dynarray_t* const dynarray, usize new_capacity);

void dynarray_push_back(dynarray_t* const dynarray, const u8 value);

inline void dynarray_push_back_buffer(dynarray_t* const dynarray, const u8* buffer, const usize size) {
    for (const u8* p = buffer; p < buffer + size; ++p) {
        dynarray_push_back(dynarray, *p);
    }
}

typedef struct static_stack_t {
    u8* data;
    size_t capacity;
    size_t size;
    usize pointer;
} static_stack_t;

inline static_stack_t static_stack_new(usize capacity) {
    static_stack_t stack = {0};

    stack.data = sigblock_malloc(capacity);
    stack.capacity = capacity;
    stack.size = 0;
    stack.pointer = capacity;

    return stack;
}

inline void static_stack_free(static_stack_t* stack) {
    sigblock_free(stack->data);
}

void static_stack_push(static_stack_t* stack, void* value, usize value_size);

u8* static_stack_pop(static_stack_t* stack, usize value_size);

inline u8* static_stack_peek(static_stack_t* stack) {
    if (stack->pointer < 0) {
        return 0;
    }

    if (stack->pointer >= stack->capacity) {
        return 0;
    }

    return stack->data + stack->pointer;
}

#ifndef ss_peek
#define ss_peek (stack, type_t) ((type_t*)(static_stack_peek((stack))))
#endif


#endif //LOOM_MEMORY_H
