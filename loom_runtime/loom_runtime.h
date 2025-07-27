#ifndef LOOM_RUNTIME_H
#define LOOM_RUNTIME_H

#include "loom_common.h"
#include "loom_memory.h"
#include "loom_scheduler.h"

#ifndef PROCESSORS_COUNT
#define PROCESSORS_COUNT 1
#endif

#ifndef WORKING_THREADS_COUNT
#define WORKING_THREADS_COUNT 1
#endif


typedef struct loom_runtime_t {
    processor_t* processors;
    thread_t* working_threads;

    pthread_t monitor;

    coroutine_queue_t global_queue;

    pthread_mutex_t queue_lock;
} loom_runtime_t;

static loom_runtime_t* loom_runtime;

void init_loom_runtime();
void free_loom_runtime();

coroutine_t* runtime_schedule(
    coroutine_func_pointer_t fn,
    usize args_count,
    usize* args_sizes,
    void* args
);
void* loom_monitor_process(void*);

#ifndef m_start_coroutine
#define m_start_coroutine(fn, ...) \
    do { \
        runtime_schedule(fn); \
    } while (0)
#endif

#endif //LOOM_RUNTIME_H
