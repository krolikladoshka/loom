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

/** coroutines **/

struct registers_t {
    u64 r[REGISTERS_COUNT]; // 0..=240
    usize sp;  // 248
    usize pc;  // 256
};

#ifndef DEFAULT_COROUTINE_STACK_SIZE
#define DEFAULT_COROUTINE_STACK_SIZE m_kilobytes(8)
#endif

struct coroutine_stack_t {
    u8* stack_pointer;

    usize pointer;
    usize base;
    usize size;
};

coroutine_stack_t* coroutine_stack_create();
void coroutine_stack_free(coroutine_stack_t* stack);

///
struct coroutine_context_t {
    registers_t registers;
    coroutine_stack_t* stack;
};

coroutine_context_t* coroutine_context_create();
void coroutine_context_free(coroutine_context_t*);
void coroutine_context_set_stack(coroutine_context_t*);

///

typedef enum coroutine_state_t {
    cs_created = 0,
    cs_runnable = 1,
    cs_running = 2,
    cs_syscall = 3, // TODO: sigmalloc&sigfree should mark this state
    cs_waiting = 4,
    cs_done = 5,
} coroutine_state_t;

struct coroutine_t {
    coroutine_context_t* context;

    any_func_pointer_t func;
    thread_t *thread;
    coroutine_state_t state;
    const char* location;
};


coroutine_t* coroutine_create(
    const char* location, any_func_pointer_t func, usize args_count, void** args
);

void coroutine_free(coroutine_t* coroutine);

/* allocate or pick cached stack, set stack pointer  */
void coroutine_prepare();

__attribute__((noinline)) volatile void* prologue(const char* location);
__attribute__((noinline)) volatile void* epilogue();
__attribute__((noinline)) volatile void yield(const char* location);

#ifndef m_start_coroutine
#define m_start_coroutine(fn, VA_ARGS) \
    do { \
        runtime_schedule(fn, ...VA_ARGS); \
    } while (0)
#endif

#ifndef m_yield
#define m_yield exit(2)
#endif


/** coroutines **/


/** coroutines queue **/

struct coroutine_queue_node_t {
    coroutine_t* coroutine;
    coroutine_queue_node_t* prev;
    coroutine_queue_node_t* next;
};

coroutine_queue_node_t* coroutine_queue_node_new(
    coroutine_queue_node_t* last, coroutine_t* coroutine
);

void coroutine_queue_node_free(coroutine_queue_node_t* node);

struct coroutine_queue_t {
    coroutine_queue_node_t* first;
    coroutine_queue_node_t* last;
    usize size;
};

void coroutine_queue_free(coroutine_queue_t* queue);
void coroutine_queue_append(coroutine_queue_t* queue, coroutine_t* coroutine);
coroutine_t* coroutine_queue_popleft(coroutine_queue_t* queue);
coroutine_t* coroutine_queue_popright(coroutine_queue_t* queue);
void coroutine_queue_reenqueue(coroutine_queue_t* queue);

/** coroutines queue **/


/** scheduler **/
struct scheduler_t {
    coroutine_t start_coroutine;

    coroutine_t* current;

    coroutine_queue_t* local_queue;
};

/** scheduler **/

struct processor_t {};


typedef struct thread_t {
    usize time_quant_start;

    scheduler_t* scheduler; // local scheduler slice
    processor_t* processor;
} thread_t;



#endif //LOOM_SCHEDULER_H
