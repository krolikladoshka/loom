#ifndef LOOM_SCHEDULER_H
#define LOOM_SCHEDULER_H

typedef struct coroutine_t {} coroutine_t;
typedef struct coroutine_context_t {} coroutine_context_t;

typedef struct scheduler_t {} scheduler_t;
typedef struct processor_t {} processor_t;
typedef struct thread_t {} thread_t;


#ifndef m_start_coroutine
#define m_start_coroutine exit(1)
#endif

#ifndef m_yield
#define m_yield exit(2)
#endif

#endif //LOOM_SCHEDULER_H
