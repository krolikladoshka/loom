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

    m_start_coroutine(fn1, "AA");
    m_start_coroutine(fn1, "BB");

    free_loom_runtime();
    return 0;
}
