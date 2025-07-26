#include "loom_runtime.h"

#include <signal.h>

#include "utils.h"


void init_loom_runtime() {
    loom_runtime = (loom_runtime_t*) malloc(
        sizeof(loom_runtime_t)
    );
    // *loom_runtime = {0};

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
    coroutine->state = cs_runnable;

    m_dev_assert(
        loom_runtime->global_queue.size == (queue_size + 1),
        "new coroutine wasn't added to global queue"
    );
    m_dev_assert(
        coroutine->state == cs_runnable,
        "coroutine state should be runnable after enqueue"
    );

    return coroutine;
}


coroutine_t* next_runnable_coroutine() {
    do {
        if (loom_runtime->global_queue.size == 0) {
            return 0;
        }

        coroutine_t* coroutine = loom_runtime->global_queue.first->coroutine;
        if (coroutine->state == cs_runnable) {
            return coroutine_queue_popleft(&loom_runtime->global_queue);
        }

        // TODO: will cycle forever if queue size == 1 & state is not runnable
        coroutine_queue_reenqueue(&loom_runtime->global_queue);
    } while (1);
}

void enqueue_to_next_thread(coroutine_t* coroutine, usize* last_received_work_thread) {
    do {
        *last_received_work_thread = (*last_received_work_thread + 1) % WORKING_THREADS_COUNT;
        thread_t* worker_thread = loom_runtime->working_threads + *last_received_work_thread;

        // TODO: check if blocked
        if (1) {
            // TODO: lock mutex
            coroutine_queue_append(worker_thread->scheduler->local_queue, coroutine);
        }
        break;
    } while (1);
}

void loom_monitor_process() {
    while (1) {
        // TODO: sleep for a quant of time
        // TODO: check threads state and business
        usize last_received_work_thread = WORKING_THREADS_COUNT - 1;

        for (usize i = 0; i < loom_runtime->global_queue.size; i++) {
            coroutine_t* coroutine = loom_runtime->global_queue.first->coroutine;

            switch (coroutine->state) {
                case cs_runnable: {
                    enqueue_to_next_thread(coroutine, &last_received_work_thread);

                    break;
                }
                case cs_done: {
                    coroutine_t* done_coroutine = coroutine_queue_popleft(
                        &loom_runtime->global_queue
                    );
                    coroutine_free(done_coroutine);

                    break;
                }
                case cs_created: {
                    m_assert(0, "unreachable state");
                    break;
                }
                case cs_running:
                case cs_syscall:
                case cs_waiting: {
                    coroutine_queue_reenqueue(&loom_runtime->global_queue);

                    break;
                }
            }
        }

        for (usize i = 0; i < WORKING_THREADS_COUNT; i++) {
            thread_t* worker_thread = loom_runtime->working_threads + i;

            if (worker_thread->scheduler->local_queue->size == 0) {
                continue;
            }

            usize now = now_ns();

            if ((now - worker_thread->time_quant_start) > m_milliseconds(10)) {
                // TODO: schedule sigurg
                int err = pthread_kill(worker_thread->pthread, SIGURG);

                m_assert(err == 0, "couldn't send SIGURG to thread");
            }
        }
    }
}
