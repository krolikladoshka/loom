#include "loom_scheduler.h"

#include <signal.h>

#include "loom_memory.h"
#include "utils.h"


extern void context_switch(
    const coroutine_t* from,
    const coroutine_t* to
) __attribute__((noreturn))
{
    restore_context(to);
}

coroutine_stack_t* coroutine_stack_create() {
    coroutine_stack_t* stack = (coroutine_stack_t*)sigblock_malloc(sizeof(coroutine_stack_t));

    m_dev_assert_null(stack, "coroutine stack create");

    *stack = {0};

    stack->pointer = 0;
    stack->base = 0;
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


coroutine_context_t* coroutine_context_create(any_func_pointer_t func) {
    coroutine_context_t* context = (coroutine_context_t*)sigblock_malloc(sizeof(coroutine_context_t));
    m_dev_assert_null(context, "coroutine context create");

    *context = {0};

    coroutine_context_set_stack(context);

    context->registers.pc = (usize)func;

    m_dev_assert(
        context->registers.sp == (usize)context->stack->stack_pointer,
        "coroutine sp register should be pointing at allocated stack"
    );
    m_dev_assert(
        context->registers.pc == (usize)func,
        "coroutine pc register should be pointing function start address"
    );

    return context;
}

void coroutine_context_set_stack(coroutine_context_t* context) {
    context->stack = coroutine_stack_create();
    m_dev_assert_null(context->stack, "coroutine stack set stack");

    context->registers.sp = (usize)context->stack->stack_pointer;

    m_dev_assert(
        context->registers.sp == (usize)context->stack->stack_pointer,
        "coroutine sp register should be pointing at allocated stack"
    );
}

void coroutine_context_free(coroutine_context_t* context) {
    m_assert_null(context, "coroutine context free");
    m_assert_null(context->stack, "coroutine stack free stack memory");

    coroutine_stack_free(context->stack);
    sigblock_free(context);
}


coroutine_t* coroutine_create(
    const char* location,
    any_func_pointer_t func,
    usize args_count,
    void** args
)
{
    m_assert_null(location, "coroutine create location");
    m_assert_null(func, "coroutine create function pointer");

    coroutine_t* coroutine = (coroutine_t*) sigblock_malloc(sizeof(coroutine_t));

    *coroutine = {0};
    coroutine->context = coroutine_context_create(func);

    m_dev_assert_null(coroutine->context, "coroutine create context");

    coroutine->func = func;
    coroutine->location = location;
    coroutine->state = cs_created;

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
    *node = {0};

    node->prev = last;
    node->coroutine = coroutine;
    node->next = 0;

    m_dev_assert(node->prev == last, "incorrect set of previous node in queue");
    m_dev_assert(node->next == 0, "next node of new node should alawys point to 0");

    return node;
}

void coroutine_queue_node_free(coroutine_queue_node_t* node) {
    m_assert_null(node, "coroutine_queue_node_free");

    sigblock_free(node);
}

///
void coroutine_queue_free(coroutine_queue_t* queue) {
    m_assert_null(queue, "coroutine_queue_free");

    coroutine_queue_node_t* current = queue->first;

    while (current != 0) {
        coroutine_queue_node_t* next = current->next;
        coroutine_queue_node_free(current);
        current = next;
    }
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

    m_dev_assert(
        pre_insert_size == 0 && queue->first == queue->last,
        "incorrect empty queue insert"
    );

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
    queue->first->prev = 0; // bet on a null pointer exception
    --queue->size;

    if (queue->size == 0) {
        queue->last = 0;
    }

#ifndef NDEBUG
    if (queue->size > 0) {
        m_dev_assert(
            queue->first->prev == 0,
            "first element's previous node should not point to anything after popleft"
        );
    } else {
        m_dev_assert(
            queue->first == 0 && queue->last == 0,
            "queue first & last shouldn't point to anything after popleft from single node queue"
        );
    }
#endif

    m_dev_assert(
        (pre_pop_size - 1) == queue->size,
        "incorrect popleft from queue"
    );

    coroutine_t* coroutine = first->coroutine;
    coroutine_queue_node_free(first);

    return coroutine;
}

coroutine_t* coroutine_queue_popright(coroutine_queue_t* queue) {
    m_assert(queue->size > 0, "can't popright from empty queue");

    coroutine_queue_node_t* last = queue->last;

    usize pre_pop_size = queue->size;
    queue->last->next = 0;
    queue->last = queue->last->prev;

    --queue->size;

    if (queue->size == 0) {
        queue->first = 0;
        queue->last = 0;
    }

#ifndef NDEBUG
    if (queue->size > 0) {
        m_dev_assert(
            queue->last->prev == 0,
            "last element's previous node should not point to anything after popright"
        );
    } else {
        m_dev_assert(
            queue->first == 0 && queue->last == 0,
            "queue first & last shouldn't point to anything after popleft from single node queue"
        );
    }
#endif

    m_dev_assert(
        (pre_pop_size - 1) == queue->size,
        "incorrect popright from queue"
    );

    coroutine_t* coroutine = last->coroutine;
    coroutine_queue_node_free(last);

    return coroutine;
}

void coroutine_queue_reenqueue(coroutine_queue_t* queue) {
    m_assert_null(queue, "coroutine queue reenqueue");

    if (queue->size == 0 || queue->size == 1) {
        return;
    }

    coroutine_queue_node_t* pre_swap_first = queue->first;

    coroutine_queue_node_t* new_first = queue->first->next;
    new_first->prev = 0;

    queue->first->next = 0;
    queue->first->prev = queue->last;
    queue->last->next = queue->first;

    queue->first = new_first;

    m_dev_assert(
        queue->last == pre_swap_first,
        "queue's last node should point to previous first element"
    );
    m_dev_assert(
        queue->first == new_first,
        "queue's new first element should be previous second"
    );
    m_dev_assert(
        queue->last->prev->next == pre_swap_first,
        "queue's last's previous node should point to previous first"
    );
}
///