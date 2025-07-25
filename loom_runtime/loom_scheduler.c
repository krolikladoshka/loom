#include "loom_scheduler.h"


extern void context_switch(
    const coroutine_t* from,
    const coroutine_t* to
) __attribute__((noreturn))
{
    restore_context(to);
}
