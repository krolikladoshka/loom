#include "loom_runtime.h"

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