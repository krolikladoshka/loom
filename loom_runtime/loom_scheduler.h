#ifndef LOOM_SCHEDULER_H
#define LOOM_SCHEDULER_H

#include <pthread.h>
#include <stdatomic.h>
#include <dispatch/dispatch.h>
#include <semaphore.h>
#include <signal.h>

#include "loom_common.h"
#include "utils.h"

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

extern void loom_runtime_save_context_arm64_darwin(
    registers_t* from
);

extern void loom_runtime_restore_context_arm64_darwin(
    registers_t* to
) __attribute((noinline, noreturn));

#ifndef loom_save_context
#define loom_save_context(x) loom_runtime_save_context_arm64_darwin((x))
#endif

#ifndef loom_restore_context
#define loom_restore_context(x) loom_runtime_restore_context_arm64_darwin((x))
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
#define DEFAULT_COROUTINE_STACK_SIZE m_kilobytes(16)
#endif

struct coroutine_stack_t {
    u8* stack_pointer;

    usize pointer;
    usize base;
    usize size;
};

coroutine_stack_t* coroutine_stack_create();
void coroutine_stack_free(coroutine_stack_t* stack);
void coroutine_stack_copy_args(
    coroutine_stack_t* stack,
    usize args_count,
    const usize* args_sizes,
    void* args
);

///
struct coroutine_context_t {
    registers_t registers;
    coroutine_stack_t* stack;
};

coroutine_context_t* coroutine_context_create(coroutine_func_pointer_t);
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

    coroutine_func_pointer_t func;
    thread_t *thread;
    _Atomic(int) state;
    const char* location;
};


coroutine_t* coroutine_create(
    const char* location,
    coroutine_func_pointer_t func,
    usize args_count,
    usize* args_sizes,
    void* args
);

void coroutine_free(coroutine_t* coroutine);

/* allocate or pick cached stack, set stack pointer  */
void coroutine_prepare();

__attribute__((noinline)) volatile void* prologue(const char* location);
__attribute__((noinline)) volatile void* epilogue();
__attribute__((noinline)) volatile void yield(const char* location);


#ifndef m_yield
#define m_yield exit(2)
#endif


/** coroutines **/


/** coroutines queue **/

struct coroutine_queue_node_t {
    coroutine_t* coroutine;
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

void coroutine_queue_init(coroutine_queue_t* queue);
void coroutine_queue_free(coroutine_queue_t* queue);
void coroutine_queue_append(coroutine_queue_t* queue, coroutine_t* coroutine);
coroutine_t* coroutine_queue_popleft(coroutine_queue_t* queue);
coroutine_t* coroutine_queue_popright(coroutine_queue_t* queue);
coroutine_t* coroutine_queue_reenqueue(coroutine_queue_t* queue);

/** coroutines queue **/


/** scheduler **/
struct scheduler_t {
    coroutine_t* current;

    coroutine_queue_t local_queue;
};

scheduler_t* scheduler_create();
void scheduler_free(scheduler_t* scheduler);

coroutine_t* scheduler_get_first_runnable(scheduler_t* scheduler);
/** scheduler **/

struct processor_t {};


typedef enum thread_state_t {
    ts_created = 0,
    ts_idle = 1,
    ts_running = 2,
    ts_scheduling = 3,
    ts_syscall = 4,
    ts_dead = 5,
} thread_state_t;

typedef struct thread_t {
    usize time_quant_start;

    scheduler_t* scheduler; // local scheduler slice
    processor_t* processor;

    coroutine_t* main_coroutine;
    // todo: why my compiler doens't recognize atomic_int?
    _Atomic(int) state;
    dispatch_semaphore_t idle_semaphore;
    // sem_t idle_semaphore;
    pthread_t pthread;
    pthread_mutex_t queue_lock;

    stack_t sighandler_stack;
} thread_t;

void sigurg_block();
void sigurg_unblock();
void copy_current_ucontext(registers_t* registers, const struct __darwin_arm_thread_state64* ss);
void sigurg_handler(int sig, siginfo_t* si, void* vp) __attribute__((noinline, noreturn));

void thread_enqueue_local(thread_t* thread, coroutine_t* coroutine);
coroutine_t* thread_popleft_local(thread_t* thread);
void thread_reenqueue_local(thread_t* thread);
usize thread_local_queue_size(thread_t* thread);

void thread_init(thread_t* thread);
void thread_free(thread_t* thread);
void* thread_schedule(void* self) __attribute__((noinline, noreturn));
void* loom_working_thread_main(void* self_raw) __attribute__((noinline, noreturn));

void thread_switch_to_coroutine(
    thread_t* thread, coroutine_t* from, coroutine_t* to
)
__attribute__((noinline, noreturn));

#endif //LOOM_SCHEDULER_H
