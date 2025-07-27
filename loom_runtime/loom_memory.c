#include "loom_common.h"
#include "loom_memory.h"

#include "loom_scheduler.h"


void def_free(void* memory) {
    free(memory);
}


void* sigblock_malloc(usize size) {
    // TODO: block signals & context switch
    sigset_t oldmask, block;
    sigemptyset(&block);
    sigaddset(&block, SIGURG);
    pthread_sigmask(SIG_BLOCK, &block, &oldmask);

    void* memory = def_malloc(size);

    pthread_sigmask(SIG_SETMASK, &oldmask, NULL);

    if (memory == 0) {
        return 0;
    }
    memset(memory, 0, size);

    return memory;
}

void sigblock_free(void* memory) {
    // TODO: block signals & context switch
    sigurg_block();
    def_free(memory);
    sigurg_unblock();
}

void* sigblock_realloc(void* memory, usize new_size) {
    return realloc(memory, new_size);
}

void memcopy(u8* to, const u8* buffer, usize buffer_size) {
    for (u8* p = to; p < (buffer + buffer_size); ++p, ++buffer) {
        *p = *buffer;
    }
}

void dynarray_extend_capacity(dynarray_t * const dynarray, usize new_capacity) {
    if (dynarray->capacity == 0) {
        dynarray->data = sigblock_malloc(new_capacity);
        dynarray->capacity = new_capacity;
    } else {
        dynarray->data = sigblock_realloc(dynarray->data, new_capacity);
        dynarray->capacity = new_capacity;
    }
}

void dynarray_push_back(dynarray_t* const dynarray, const u8 value) {
    static const size_t default_dynarray_size = 8;
    static const size_t growth_size_cap = m_megabytes(512);

    // todo: if dynarray->size + size <= dynarray->capacity * 2 is not handled
    if (dynarray->capacity == 0 || dynarray->size >= dynarray->capacity) {
        size_t new_capacity = default_dynarray_size;

        if (dynarray->capacity < default_dynarray_size) {
            new_capacity = default_dynarray_size;
        } else if (dynarray->capacity < growth_size_cap) {
            new_capacity = dynarray->capacity * 2;
        } else {
            new_capacity = growth_size_cap;
        }

        dynarray_extend_capacity(dynarray, new_capacity);
    }

    dynarray->data[dynarray->size] = value;
    ++dynarray->size;
}

/* static stack */

void static_stack_push(static_stack_t *stack, void *value, usize value_size) {
    if (stack->pointer - value_size < 0) {
        return; // TODO: error stack overflow
    }

    stack->pointer -= value_size;
    memcopy(stack->data + stack->pointer, value, value_size);
    stack->size += 1;
}

inline u8* static_stack_pop(static_stack_t *stack, usize value_size) {
    if (stack->pointer + value_size > stack->capacity) {
        return 0; // TODO: pop empty stack || pop beyond the stack
    }

    u8* value = stack->data + stack->pointer;
    stack->pointer += value_size;

    return value;
}
