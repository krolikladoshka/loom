#include "loom_scheduler.h"

#include <setjmp.h>
#include <signal.h>
#include <stdio.h>
#include <sys/ucontext.h>
#include "loom_memory.h"
#include "utils.h"

_Thread_local thread_t* tls_self;


extern void context_switch(
    const coroutine_t* from,
    const coroutine_t* to
) __attribute__((noreturn))
{
    // restore_context(to);
}

coroutine_stack_t* coroutine_stack_create() {
    coroutine_stack_t* stack = (coroutine_stack_t*)sigblock_malloc(sizeof(coroutine_stack_t));

    m_dev_assert_null(stack, "coroutine stack create");

    // *stack = {0};

    stack->pointer = DEFAULT_COROUTINE_STACK_SIZE;
    stack->base = DEFAULT_COROUTINE_STACK_SIZE;
    stack->size = DEFAULT_COROUTINE_STACK_SIZE;
    stack->stack_pointer = (u8*)sigblock_malloc(stack->size);

    m_dev_assert(stack->stack_pointer, "coroutine stack memory alloc");

    return stack;
}

void coroutine_stack_free(coroutine_stack_t* stack) {
    m_assert_null(stack, "coroutine stack free");

    // TODO: save self pointer to cache/mark it as free for further reuse by other coroutines
    m_dev_assert(stack->stack_pointer, "coroutine stack allocated memory free");

    sigblock_free(stack->stack_pointer);
    sigblock_free(stack);
}

void coroutine_stack_push(coroutine_stack_t* stack, u8 value) {
    --stack->pointer;
    stack->stack_pointer[stack->pointer] = value;
}

void coroutine_stack_copy_args(
    coroutine_stack_t* stack,
    usize args_count,
    const usize* args_sizes,
    void* args
)
{
    stack->base = stack->pointer;

    usize args_pointer = 0;
    for (i64 i = (i64)args_count - 1; i >= 0; i--) {
        usize arg_size = args_sizes[i];
        for (i64 j = arg_size - 1; j >= 0; j--) {
            u8 byte = *(((u8*)args) + args_pointer + j);
            coroutine_stack_push(stack, byte);
            // *(stack->stack_pointer + stack->pointer + j) = *(((u8*)(args)) + args_pointer + j);
        }
        // stack->pointer += arg_size;
        args_pointer += arg_size;
    }
}


coroutine_context_t* coroutine_context_create(coroutine_func_pointer_t func) {
    coroutine_context_t* context = (coroutine_context_t*)sigblock_malloc(sizeof(coroutine_context_t));
    m_dev_assert_null(context, "coroutine context create");

    // *context = {0};

    coroutine_context_set_stack(context);

    context->registers.pc = (usize)func;

    // m_dev_assert(
        // context->registers.sp == (usize)(context->stack->stack_pointer + context->stack->size),
        // "coroutine sp register should be pointing at allocated stack"
    // );
    m_dev_assert(
        context->registers.pc == (usize)func,
        "coroutine pc register should be pointing function start address"
    );

    return context;
}

void coroutine_context_set_stack(coroutine_context_t* context) {
    context->stack = coroutine_stack_create();
    m_dev_assert_null(context->stack, "coroutine stack set stack");

    context->registers.sp = (usize)(context->stack->stack_pointer + context->stack->size); // & ~0xF;

    // m_dev_assert(
        // context->registers.sp == (usize)(context->stack->stack_pointer + context->stack->size),
        // "coroutine sp register should be pointing at allocated stack"
    // );
}

void coroutine_context_free(coroutine_context_t* context) {
    m_assert_null(context, "coroutine context free");
    m_assert_null(context->stack, "coroutine stack free stack memory");

    coroutine_stack_free(context->stack);
    sigblock_free(context);
}

void coroutine_context_copy_args(
    coroutine_context_t* context,
    usize args_count,
    usize* args_sizes,
    void* args
)
{
    usize args_pointer = 0;
    for (usize i = 0; i < args_count; i++) {
        usize arg_size = args_sizes[i];
        switch (arg_size) {
            case 1: {
                context->registers.r[i] = *(((u8*)args) + args_pointer);
                break;
            }
            case 2: {
                context->registers.r[i] = *(u16*)(((u8*)args) + args_pointer);
                break;
            }
            case 4: {
                context->registers.r[i] = *(u32*)(((u8*)args) + args_pointer);
                break;
            }
            case 8: {
                context->registers.r[i] = *(u64*)(((u8*)args) + args_pointer);
                break;
            }
            default: {
                m_assert(0, "Unsupported arg size . . . for now");
                return;
            }
        }
        // context->registers.r[i] = *(usize*)((u8*)args + args_pointer);
        args_pointer += args_sizes[i];
    }
}
coroutine_t* coroutine_create(
    const char* location,
    coroutine_func_pointer_t func,
    usize args_count,
    usize* args_sizes,
    void* args
)
{
    m_assert_null(location, "coroutine create location");
    m_assert_null(func, "coroutine create function pointer");

    coroutine_t* coroutine = (coroutine_t*) sigblock_malloc(sizeof(coroutine_t));

    // *coroutine = {0};
    coroutine->context = coroutine_context_create(func);

    coroutine_context_copy_args(coroutine->context, args_count, args_sizes, args);
    // coroutine_stack_copy_args(coroutine->context->stack, args_count, args_sizes, args);
    m_dev_assert_null(coroutine->context, "coroutine create context");

    coroutine->func = func;
    coroutine->location = location;

    __c11_atomic_store(&coroutine->state, cs_created, __ATOMIC_RELAXED);

    return coroutine;
}

void coroutine_free(coroutine_t* coroutine) {
    m_assert_null(coroutine, "coroutine free");
    m_assert_null(coroutine->context, "coroutine context free");

    coroutine_context_free(coroutine->context);
    sigblock_free(coroutine);
}

///////

coroutine_queue_node_t* coroutine_queue_node_new(
    coroutine_queue_node_t* last, coroutine_t* coroutine
)
{
    coroutine_queue_node_t* node = (coroutine_queue_node_t*)sigblock_malloc(
        sizeof(coroutine_queue_node_t)
    );
    // *node = {0};

    node->coroutine = coroutine;
    node->next = 0;

    if (last != 0) {
        last->next = node;
        m_dev_assert(last->next == node, "incorrect set of previous node in queue");
    }
    m_dev_assert(node->next == 0, "next node of new node should alawys point to 0");

    return node;
}

void coroutine_queue_node_free(coroutine_queue_node_t* node) {
    m_assert_null(node, "coroutine_queue_node_free");
    // coroutine_free(node->coroutine);
    sigblock_free(node);
}

///
void coroutine_queue_init(coroutine_queue_t* queue) {
    queue->first = 0;
    queue->last = 0;

    queue->size = 0;
}

void coroutine_queue_free(coroutine_queue_t* queue) {
    m_assert_null(queue, "coroutine_queue_free");

    coroutine_queue_node_t* current = queue->first;

    while (current != 0) {
        coroutine_queue_node_t* next = current->next;
        coroutine_queue_node_free(current);
        current = next;
    }

    // sigblock_free(queue);
}

void coroutine_queue_append(coroutine_queue_t* queue, coroutine_t* coroutine) {
    m_assert_null(coroutine, "coroutine queue append");

    usize pre_insert_size = queue->size;

    coroutine_queue_node_t* last = queue->last;
    coroutine_queue_node_t* new_node = coroutine_queue_node_new(last, coroutine);

    queue->last = new_node;

    if (pre_insert_size == 0) {
        queue->first = queue->last;
    }

#ifndef NDEBUG
    if (queue->size == 0) {
        m_dev_assert(
            queue->first == queue->last,
            "incorrect empty queue insert"
        );
    }
#endif

    ++queue->size;

    m_dev_assert(
        (pre_insert_size + 1) == queue->size,
        "coroutine wasn't added to queue"
    );
}

coroutine_t* coroutine_queue_popleft(coroutine_queue_t* queue) {
    m_assert(queue->size > 0, "can't popleft from empty queue");

    coroutine_queue_node_t* first = queue->first;

    usize pre_pop_size = queue->size;
    queue->first = first->next;
    --queue->size;

    if (queue->size == 0) {
        queue->last = 0;
    }

#ifndef NDEBUG
    // if (queue->size > 0) {
    //     m_dev_assert(
    //         queue->first->prev == 0,
    //         "first element's previous node should not point to anything after popleft"
    //     );
    // } else {
    if (queue->size == 0) {
        m_dev_assert(
            queue->first == 0 && queue->last == 0,
            "queue first & last shouldn't point to anything after popleft from single node queue"
        );
    }
    // }
#endif

    m_dev_assert(
        (pre_pop_size - 1) == queue->size,
        "incorrect popleft from queue"
    );

    coroutine_t* coroutine = first->coroutine;
    coroutine_queue_node_free(first);

    return coroutine;
}

// coroutine_t* coroutine_queue_popright(coroutine_queue_t* queue) {
//     m_assert(queue->size > 0, "can't popright from empty queue");
//
//     coroutine_queue_node_t* last = queue->last;
//
//     usize pre_pop_size = queue->size;
//     queue->last->next = 0;
//     queue->last = queue->last->prev;
//
//     --queue->size;
//
//     if (queue->size == 0) {
//         queue->first = 0;
//         queue->last = 0;
//     }
//
// #ifndef NDEBUG
//     if (queue->size > 0) {
//         m_dev_assert(
//             queue->last->prev == 0,
//             "last element's previous node should not point to anything after popright"
//         );
//     } else {
//         m_dev_assert(
//             queue->first == 0 && queue->last == 0,
//             "queue first & last shouldn't point to anything after popleft from single node queue"
//         );
//     }
// #endif
//
//     m_dev_assert(
//         (pre_pop_size - 1) == queue->size,
//         "incorrect popright from queue"
//     );
//
//     coroutine_t* coroutine = last->coroutine;
//     coroutine_queue_node_free(last);
//
//     return coroutine;
// }

coroutine_t* coroutine_queue_reenqueue(coroutine_queue_t* queue) {
    m_assert_null(queue, "coroutine queue reenqueue");

    if (queue->size == 0) {
        return 0;
    }
    if (queue->size == 1) {
        return queue->first->coroutine;
    }

    coroutine_queue_node_t* pre_swap_first = queue->first;

    coroutine_queue_node_t* new_first = queue->first->next;

    queue->first->next = 0;
    queue->last->next = queue->first;
    queue->last = queue->first;
    queue->first = new_first;

    m_dev_assert(
        queue->last == pre_swap_first,
        "queue's last node should point to previous first element"
    );
    m_dev_assert(
        queue->first == new_first,
        "queue's new first element should be previous second"
    );
    // m_dev_assert(
    //     queue->last->prev->next == pre_swap_first,
    //     "queue's last's previous node should point to previous first"
    // );

    return queue->first->coroutine;
}

///
/// scheduler_t
///

scheduler_t* scheduler_create() {
    scheduler_t* scheduler = (scheduler_t*)sigblock_malloc(sizeof(scheduler_t));

    scheduler->current = 0;
    coroutine_queue_init(&scheduler->local_queue);
    // scheduler->local_queue = {0};

    return scheduler;
}

void scheduler_free(scheduler_t* scheduler) {
    coroutine_queue_free(&scheduler->local_queue);
    sigblock_free(scheduler);
}

coroutine_t* scheduler_get_first_runnable(scheduler_t* scheduler) {
    for (usize i = 0; i < scheduler->local_queue.size; i++) {
        coroutine_t* coroutine = scheduler->local_queue.first->coroutine;
        m_dev_assert_null(coroutine, "coroutine queue should be empty");

        if (__c11_atomic_load(&coroutine->state, __ATOMIC_SEQ_CST) == cs_runnable) {
            scheduler->current = coroutine;

            // coroutine_queue_reenqueue(&scheduler->local_queue);

            return coroutine;
        }

        if (__c11_atomic_load(&coroutine->state, __ATOMIC_SEQ_CST) == cs_done) {
            coroutine = coroutine_queue_popleft(&scheduler->local_queue);
            coroutine_free(coroutine);
            coroutine = 0;
            scheduler->current = 0;
        }

        coroutine_queue_reenqueue(&scheduler->local_queue);
    }

    scheduler->current = 0;

    return 0;
}


///
/// thread_t
///

void sigurg_block() {
    sigset_t set;
    sigemptyset(&set);
    sigaddset(&set, SIGURG);
    pthread_sigmask(SIG_BLOCK, &set, 0);
}

void sigurg_unblock() {
    sigset_t set;
    sigemptyset(&set);
    sigaddset(&set, SIGURG);
    pthread_sigmask(SIG_UNBLOCK, &set, 0);
}

void thread_enqueue_local(thread_t* thread, coroutine_t* coroutine) {
    pthread_mutex_lock(&thread->queue_lock);
    coroutine_queue_append(&thread->scheduler->local_queue, coroutine);
    pthread_mutex_unlock(&thread->queue_lock);
}

coroutine_t* thread_popleft_local(thread_t* thread) {
    pthread_mutex_lock(&thread->queue_lock);
    coroutine_t* coroutine = coroutine_queue_popleft(&thread->scheduler->local_queue);
    pthread_mutex_unlock(&thread->queue_lock);

    return coroutine;
}

void thread_reenqueue_local(thread_t* thread) {
    pthread_mutex_lock(&thread->queue_lock);
    coroutine_queue_reenqueue(&thread->scheduler->local_queue);
    pthread_mutex_unlock(&thread->queue_lock);
}

usize thread_local_queue_size(thread_t* thread) {
    pthread_mutex_lock(&thread->queue_lock);
    usize size = thread->scheduler->local_queue.size;
    pthread_mutex_unlock(&thread->queue_lock);

    return size;
}


void thread_init(thread_t* thread) {
    thread->scheduler = scheduler_create();
    thread->processor = 0;

    thread->time_quant_start = now_ns();
    thread->state = ts_created;

    usize arg_sizes[1] = {sizeof(thread_t*)};
    void* args = {thread};
    thread->main_coroutine = coroutine_create(
        "THREAD_MONITOR",
        thread_schedule,
        1,
        arg_sizes,
        &thread
    );
    thread->idle_semaphore = dispatch_semaphore_create(0);
    m_assert(pthread_mutex_init(&thread->queue_lock, 0) == 0, "couldn't create mutex");

    thread->sighandler_stack.ss_flags = 0;
    thread->sighandler_stack.ss_sp = sigblock_malloc(m_kilobytes(32));
    thread->sighandler_stack.ss_size = m_kilobytes(32);
    sigaltstack(&thread->sighandler_stack, NULL);

    struct sigaction sa = {0};
    sa.sa_sigaction = sigurg_handler;
    sa.sa_flags = SA_SIGINFO | SA_ONSTACK;
    sigemptyset(&sa.sa_mask);
    sigaction(SIGURG, &sa, NULL);

    // sem_init(&thread->idle_semaphore, 0, 0);
    pthread_create(&thread->pthread, 0, &loom_working_thread_main, thread);
}

void thread_free(thread_t* thread) {
    // TODO: no gracefull cleanup for now
    m_assert(pthread_mutex_destroy(&thread->queue_lock) != 0, "couldn't destroy mutex");
    sigblock_free(thread->sighandler_stack.ss_sp);
    pthread_kill(thread->pthread, SIGKILL);

    coroutine_free(thread->main_coroutine);
    scheduler_free(thread->scheduler);
}

void thread_switch_to_coroutine(thread_t* thread, coroutine_t* from, coroutine_t* to)
__attribute__((noinline, noreturn))
{
    thread->scheduler->current = to;

    // loom_save_context(to->context->registers);
    loom_restore_context(&to->context->registers);
}

inline void copy_current_ucontext(registers_t* registers, const struct __darwin_arm_thread_state64* ss) {
    // memcpy(&registers->r, &ss->__x, 29); // todo: ebat chto? for some reason doesn't work and misses some of registers
    for (usize i = 0; i < 29; i++) {
        registers->r[i] = ss->__x[i];
    }
    registers->r[29] = ss->__fp;
    registers->r[30] = ss->__lr;

    registers->sp = ss->__sp;;
    registers->pc = ss->__pc;
}

void sigurg_handler(int sig, siginfo_t* si, void* vp) {
    (void)sig; (void)si;
    thread_t* thread = tls_self;
    __c11_atomic_store(&thread->state, ts_scheduling, __ATOMIC_SEQ_CST);
    sigurg_block();

    ucontext_t* ucontext = (ucontext_t*) vp;

    coroutine_t* current_coroutine = thread->scheduler->current;
    copy_current_ucontext(&current_coroutine->context->registers, &ucontext->uc_mcontext->__ss);
    __c11_atomic_store(&current_coroutine->state, cs_runnable, __ATOMIC_SEQ_CST);

    thread_reenqueue_local(thread);

    // TODO Cyclic recursion 3: jump to scheduler
    loom_restore_context(&thread->main_coroutine->context->registers);
}

void* thread_schedule(void* self_raw) __attribute__((noinline, noreturn)) {
    thread_t* self = (thread_t*)self_raw;
    sigurg_block();

    __c11_atomic_store(&self->state, ts_scheduling, __ATOMIC_SEQ_CST);

    if (thread_local_queue_size(self) < 0) {
        // TODO Cyclic recursion 4 jump to main thread handler and sleep?
        __c11_atomic_store(&self->state, ts_idle, __ATOMIC_SEQ_CST);
        loom_working_thread_main(self);
    }

    pthread_mutex_lock(&self->queue_lock);

    if (self->scheduler->current && self->scheduler->current->state == cs_running) {
        self->scheduler->current->state = cs_runnable;
        coroutine_queue_reenqueue(&self->scheduler->local_queue);
    }

    // pthread_mutex_unlock(&self->queue_lock);

    // pthread_mutex_lock(&self->queue_lock);
    coroutine_t* coroutine = scheduler_get_first_runnable(self->scheduler);
    self->scheduler->current = coroutine;

    pthread_mutex_unlock(&self->queue_lock);

    if (coroutine == 0) {
        // TODO Cyclic recursion 4: jump to main thread handler and sleep?
        __c11_atomic_store(&self->state, ts_idle, __ATOMIC_SEQ_CST);
        loom_working_thread_main(self);
    }

    self->time_quant_start = now_ns();
    __c11_atomic_store(&coroutine->state, cs_running, __ATOMIC_SEQ_CST);
    __c11_atomic_store(&self->state, ts_running, __ATOMIC_SEQ_CST);
    sigurg_unblock();

    // TODO Cyclic recursion 2: jump to coroutine
    loom_restore_context(&coroutine->context->registers);
}


void* loom_working_thread_main(void* self_raw)
    __attribute__((noinline, noreturn))
{
    // TODO Cyclic recursion 0: enter the thread
    // TODO Cyclic recursion 5: jump from scheduler to sleep?
    sigurg_block();

    thread_t* self = (thread_t*) self_raw;
    tls_self = self;

    __c11_atomic_store(&self->state, ts_created, __ATOMIC_SEQ_CST);

    while (1) {
        sigurg_block();
        // __atomic_store()
        __c11_atomic_store(&self->state, ts_idle, __ATOMIC_SEQ_CST);
        dispatch_semaphore_wait(self->idle_semaphore, DISPATCH_TIME_FOREVER);

        if (thread_local_queue_size(self) < 0) {
            continue;
        }

        // pthread_mutex_lock(&self->queue_lock);
        // coroutine_t* coroutine = scheduler_get_first_runnable(self->scheduler);
        // pthread_mutex_unlock(&self->queue_lock);

        // if (coroutine == 0) {
            // continue;
        // }
        // __c11_atomic_store(&self->state, ts_running, __ATOMIC_SEQ_CST);
        // TODO Cyclic recursion 1: call thread_schedule
        // loom_restore_context(&coroutine->context->registers);
        loom_restore_context(&self->main_coroutine->context->registers);
        // thread_schedule(self_raw);
    }

    return 0;
}