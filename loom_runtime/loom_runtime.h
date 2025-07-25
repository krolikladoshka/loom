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
    thread_t* monitor;

    coroutine_queue_t global_queue;
} loom_runtime_t;

static loom_runtime_t* loom_runtime;

void init_loom_runtime();
void free_loom_runtime();



#endif //LOOM_RUNTIME_H
