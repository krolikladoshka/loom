#include "loom_runtime.h"

#include <signal.h>
#include <stdio.h>
#include <unistd.h>

#include "utils.h"


void init_loom_runtime() {
    loom_runtime = (loom_runtime_t*) malloc(
        sizeof(loom_runtime_t)
    );
    coroutine_queue_init(&loom_runtime->global_queue);
    // *loom_runtime = {0};
    m_assert(pthread_mutex_init(&loom_runtime->queue_lock, 0) == 0, "couldn't create mutex");

    loom_runtime->processors = (processor_t*) malloc(
        sizeof(processor_t) * PROCESSORS_COUNT
    );

    loom_runtime->working_threads = (thread_t*) malloc(
        sizeof(thread_t) * WORKING_THREADS_COUNT
    );

    for (int i = 0; i < WORKING_THREADS_COUNT; i++) {
        thread_t* thread = &loom_runtime->working_threads[i];
        thread_init(thread);
    }

    pthread_create(&loom_runtime->monitor, 0, loom_monitor_process, 0);
}

void free_loom_runtime() {
    if (loom_runtime == 0) {
        return;
    }
    m_assert(pthread_mutex_destroy(&loom_runtime->queue_lock) == 0, "couldn't destroy mutex");
    pthread_kill(loom_runtime->monitor, SIGKILL);

    for (usize i = 0; i < WORKING_THREADS_COUNT; i++) {
        thread_free(&loom_runtime->working_threads[i]);
    }
    free(loom_runtime->working_threads);


    for (usize i = 0; i < PROCESSORS_COUNT; i++) {
        free(loom_runtime->processors + i);
    }

    coroutine_queue_free(&loom_runtime->global_queue);
}

coroutine_t* runtime_schedule(
    coroutine_func_pointer_t func,
    usize args_count,
    usize* args_sizes,
    void* args
) {
    sigurg_block();

    coroutine_t* coroutine = coroutine_create(
        __FILE__,
        func,
        args_count,
        args_sizes,
        args
    );

    m_assert_null(coroutine, "runtime schedule coroutine_create");

    pthread_mutex_lock(&loom_runtime->queue_lock);

    usize queue_size = loom_runtime->global_queue.size;

    coroutine_queue_append(&loom_runtime->global_queue, coroutine);
    __c11_atomic_store(&coroutine->state, cs_runnable, __ATOMIC_SEQ_CST);

    m_dev_assert(
        loom_runtime->global_queue.size == (queue_size + 1),
        "new coroutine wasn't added to global queue"
    );
    m_dev_assert(
        coroutine->state == cs_runnable,
        "coroutine state should be runnable after enqueue"
    );

    pthread_mutex_unlock(&loom_runtime->queue_lock);
    sigurg_unblock();

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
            thread_enqueue_local(worker_thread, coroutine);
            // coroutine_queue_append(&worker_thread->scheduler->local_queue, coroutine);
        }
        break;
    } while (1);
}

void recycle_global_queue(usize* last_received_work_thread) {
    pthread_mutex_lock(&loom_runtime->queue_lock);

    for (usize i = 0; i < loom_runtime->global_queue.size; i++) {
        m_dev_assert_null(loom_runtime->global_queue.first, "global queue is corrupted");

        coroutine_t* coroutine = loom_runtime->global_queue.first->coroutine;

        switch (__c11_atomic_load(&coroutine->state, __ATOMIC_SEQ_CST)) {
            case cs_runnable: {
                enqueue_to_next_thread(coroutine, last_received_work_thread);
                coroutine_queue_popleft(&loom_runtime->global_queue);
                break;
            }
            case cs_done: {
                coroutine_queue_popleft(
                    &loom_runtime->global_queue
                );
                coroutine_free(coroutine);
                coroutine = 0;

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
            default: {
                m_assert(0, "unreachable state");
                return;
            };
        }
    }

    pthread_mutex_unlock(&loom_runtime->queue_lock);
}

void* loom_monitor_process(void*) {
    usize sleep_interval = 0;
    usize last_received_work_thread = WORKING_THREADS_COUNT - 1;

    while (1) {
        usleep(sleep_interval);

        for (usize i = 0; i < WORKING_THREADS_COUNT; i++) {
            thread_t* worker_thread = loom_runtime->working_threads + i;

            recycle_global_queue(&last_received_work_thread);

            if (thread_local_queue_size(worker_thread) == 0) {
            // if (worker_thread->scheduler->local_queue.size == 0) {
                continue;
            }

            switch (__c11_atomic_load(&worker_thread->state, __ATOMIC_SEQ_CST)) {
                case ts_created: {
                    continue;
                }
                case ts_idle: {
                    dispatch_semaphore_signal(worker_thread->idle_semaphore);
                    continue; // should it?
                }
                case ts_running: {
                    usize now = now_ns();

                    // todo: atomically exchange start time
                    if ((__c11_atomic_load(&worker_thread->state, __ATOMIC_SEQ_CST) == ts_running) &&
                        (now - worker_thread->time_quant_start) > m_milliseconds(20))
                    {
                        int err = pthread_kill(worker_thread->pthread, SIGURG);
                        m_assert(err == 0, "couldn't send SIGURG to thread");
                    }
                    break;
                }
                default: {}
            }
        }

        // 1 second
        sleep_interval = 500;
    }
}
