#ifndef LOOM_SCHEDULER_H
#define LOOM_SCHEDULER_H
#include "loom_common.h"

typedef struct thread_t thread_t;
typedef struct scheduler_t scheduler_t;
typedef struct processor_t processor_t;

typedef struct registers_t registers_t;
typedef struct coroutine_context_t coroutine_context_t;
typedef struct coroutine_stack_t coroutine_stack_t;
typedef struct coroutine_t coroutine_t;

typedef struct coroutine_queue_node_t coroutine_queue_node_t;
typedef struct coroutine_queue_t coroutine_queue_t;


#ifndef REGISTERS_COUNT
#define REGISTERS_COUNT 31
#endif

extern void loom_runtime_restore_context_arm64_darwin(
    coroutine_context_t* to
) __attribute((noinline));

#ifndef restore_context
#define restore_context(x) loom_runtime_restore_context_arm64_darwin((x))
#endif

// inline void restore_context(coroutine_context_t* to) __attribute__((noreturn)) {
    // loom_runtime_restore_context_arm64_darwin(to);
// }

extern void context_switch(
    const coroutine_t* from,
    const coroutine_t* to
) __attribute__((noreturn));


struct registers_t {
    u64 r[REGISTERS_COUNT]; // 0..=240
    usize sp;  // 248
    usize pc;  // 256
};

struct coroutine_stack_t {
    usize* stack_pointer;

    usize base;
    usize size;
};

struct coroutine_context_t {
    registers_t registers;
    coroutine_stack_t* stack;
};

struct coroutine_t {
    coroutine_context_t context;

    thread_t *thread;
    const char* location;
};

struct coroutine_queue_node_t {
    coroutine_t* coroutine;
    coroutine_t* prev;
    coroutine_t* next;
};

struct coroutine_queue_t {
    coroutine_queue_node_t* first;
    coroutine_queue_node_t* last;
    usize size;
};

struct scheduler_t {
    coroutine_t start_coroutine;

    coroutine_t* current;

    coroutine_queue_node_t* first;
    coroutine_queue_node_t* last;
};

struct processor_t {};


typedef struct thread_t {
    usize time_quant_start;

    scheduler_t* scheduler; // local scheduler slice
    processor_t* processor;
} thread_t;


__attribute__((noinline)) volatile void* prologue(const char* location);
__attribute__((noinline)) volatile void* epilogue();
__attribute__((noinline)) volatile void yield(const char* location);

#ifndef m_start_coroutine
#define m_start_coroutine(fn, VA_ARGS) \
    do { \
        exit(1); \
        loom_runtime->schedule(fn, ...VA_ARGS); \
    } while (0);
#endif

#ifndef m_yield
#define m_yield exit(2)
#endif

#endif //LOOM_SCHEDULER_H
