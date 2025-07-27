#include <stdio.h>
#include <unistd.h>

#include "loom_runtime.h"

volatile void simulate_work() {
    for (volatile usize i = 0; i < 5000000; i++);
}

int calculate(int i) {
    return i * 10;
}

void* fn1(void* _) {
    int j = 0;
    char str[100] = {0};

    while (1) {
        int calc = calculate(j++);
        sigurg_block();
        // puts((char*)_);
        // puts(" ");

        sprintf(str, "%s %d\n", (char*)_, calc);
        puts(str);
        // fprintf("%s %ul", (char*)_, calc);
        sigurg_unblock();
        simulate_work();
    }
}

void* fn2(void* _) {
    while (1) {
        sigurg_block();
        puts("B");
        sigurg_unblock();
        simulate_work();
    }
}

void* fn3(void* _) {
    while (1) {
        sigurg_block();
        puts("C");
        sigurg_unblock();
        simulate_work();
    }
}

u64 next_fibb(u64 prev, u64 current) __attribute__((noinline)) {
    return prev + current;
}

void* fibb(void* _) {
    char message[2048] = {0};
    u64 prev = 0;
    u64 current = 1;

    for (usize i = 0;; i++) {
        u64 next = next_fibb(prev, current);
        prev = current;
        current = next;
        sprintf(message, "%s: %lluth=%llu", (char*)_, i, next);
        sigurg_block();
        puts(message);
        sigurg_unblock();
        memset(message, 0, 2048);
        simulate_work();
    }
}

void* factorial(void*_) {
    char message[2048] = {0};
    u64 fc = 1;
    for (u64 i = 1;; i++) {
        if (i == 0) {
            i = 1;
            fc = 1;
        }
        sprintf(message, "%s: %lluth=%llu", (char*)_, i, fc);
        sigurg_block();
        puts(message);
        sigurg_unblock();

        fc *= i;

        if (fc == 0) {
            fc = 1;
        }

        simulate_work();
    }
}

u64 calc_ackermann_fn(u64 n, u64* stack, usize stack_size) {
    u64 m = n;
    u64 k = n;
    u64 top = 0;

    while (1) {
        if (m == 0) {
            k++;

            if (top == 0) {
                return k;
            }
            m = stack[--top];
        } else if (k == 0) {
            m--;
            k = 1;
        } else {
            stack[top++] = m - 1;
            k--;
        }
    }
}
void* ackermann_function(void*_) {
    char message[m_kilobytes(2)] = {0};
    u64* stack = (u64*)sigblock_malloc(m_megabytes(16));
    for (usize i = 0;; i++) {
        u64 ack = calc_ackermann_fn(i, stack, m_megabytes(2048) >> 3);

        sprintf(message, "%s: %lluth=%llu", (char*)_, i, ack);
        sigurg_block();
        puts(message);
        sigurg_unblock();
    }
    sigblock_free(stack);
}

int main() {
    init_loom_runtime();
    /**
     * TODO: well it seems that trampoline is mandatory
     * TODO: temporary solution?: cwrapper: c->state = running; for (;;) { if c->state = running {fn();} c->state = done; }
     * TODO: ^won't work
     * TODO: trampoline(sched, fn): save_context(sched); restore_context; call tramp -> call restore_context -> call fn -> return to scheduler
     **/

    const char* aa = "AA";
    const char* bb = "I'm a fn1 second go call";;
    const char* cc = "Im a fn1 third go call";
    const char* ff = "I endlessly calculate fibb numbers";
    const char* ff2 = "Calculating factorial";
    const char* ff3 = "Calculating ackermann function";
    // printf("%s %lu", aa, sizeof(aa));
    usize sizes[] = { sizeof(aa) };
    void* args_a[] = {&aa};
    void* args_b[] = {&bb};
    void* args_c[] = {&bb};

    runtime_schedule(fn1, 1, sizes, &aa);
    runtime_schedule(fn1, 1, sizes, &bb);
    runtime_schedule(fn1, 1, sizes, &cc);
    runtime_schedule(fn2, 1, sizes, &bb);
    runtime_schedule(fn3, 1, sizes, &cc);
    runtime_schedule(fibb, 1, sizes, &ff);
    runtime_schedule(factorial, 1, sizes, &ff2);
    runtime_schedule(ackermann_function, 1, sizes, &ff3);


    usleep(m_seconds(300) / m_microseconds(1));
    free_loom_runtime();
    return 0;
}
