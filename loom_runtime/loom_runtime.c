#include "loom_runtime.h"

#include "utils.h"

void init_loom_runtime() {
    loom_runtime = {0};

    loom_runtime = (loom_runtime_t*) malloc(
        sizeof(loom_runtime_t)
    );

    loom_runtime->processors = (processor_t*) malloc(
        sizeof(processor_t*) * PROCESSORS_COUNT
    );

    loom_runtime->working_threads = (thread_t*) malloc(
        sizeof(thread_t*) * WORKING_THREADS_COUNT
    );
    loom_runtime->monitor = (thread_t*) malloc(
        sizeof(thread_t)
    );
}

void free_loom_runtime() {
    if (loom_runtime == 0) {
        return;
    }

    for (usize i = 0; i < WORKING_THREADS_COUNT; i++) {
        free(loom_runtime->working_threads + i);
    }

    free(loom_runtime->monitor);

    for (usize i = 0; i < PROCESSORS_COUNT; i++) {
        free(loom_runtime->processors + i);
    }
}

coroutine_t* runtime_schedule(any_func_pointer_t func) {
    coroutine_t* coroutine = coroutine_create(
        __FILE__,
        func,
        0,
        0
    );

    m_assert_null(coroutine, "runtime schedule coroutine_create");

    usize queue_size = loom_runtime->global_queue.size;
    // TODO: lock mutex?
    coroutine_queue_append(&loom_runtime->global_queue, coroutine);

    m_dev_assert(
        loom_runtime->global_queue.size == (queue_size + 1),
        "new coroutine wasn't added to global queue"
    );

    return coroutine;
}

