#include "utils.h"

#include <_time.h>

#include "loom_common.h"

usize now_ns() {
    struct timespec ts;

    clock_gettime(CLOCK_MONOTONIC, &ts);

    return m_seconds(ts.tv_sec) + (usize)ts.tv_nsec;
}


