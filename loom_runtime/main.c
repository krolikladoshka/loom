#include <stdio.h>

#include "loom_runtime.h"


void fn1(const char* name) {
    while (1) {
        printf(name);
        for (volatile int i = 0; i < 500000000; i++);
    }
}


int main() {
    init_loom_runtime();
    /**
     * TODO: well it seems that trampoline is mandatory
     * TODO: temporary solution?: cwrapper: c->state = running; for (;;) { if c->state = running {fn();} c->state = done; }
     * TODO: ^won't work
     * TODO: trampoline(sched, fn): save_context(sched); restore_context; call tramp -> call restore_context -> call fn -> return to scheduler
     **/
    // m_start_coroutine(fn1, "AA");
    // m_start_coroutine(fn1, "BB");

    free_loom_runtime();
    return 0;
}
