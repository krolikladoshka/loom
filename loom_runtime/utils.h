#ifndef UTILS_H
#define UTILS_H

#include <assert.h>

#include "loom_common.h"

#ifndef m_assert
#define m_assert(condition, message) \
    do { assert(((condition) && (message))); } while (0)
#endif

#ifndef m_assert_null
#define m_assert_null(pointer, operation) \
    do { m_assert(((pointer) != 0), ("can't perform " operation " on null")); } while (0)
#endif


#ifndef NDEBUG

#define m_dev_assert(condition, message) \
    do { assert((condition) && (message)); } while (0)

#define m_dev_assert_null(pointer, operation) \
    m_dev_assert(((pointer) != 0), ("can't perform " operation " on null"))

#else

#define m_dev_assert(condition, message) ((void*)0)
#define m_dev_assert(pointer, operation) ((void*)0)

#endif


#ifndef m_seconds
#define m_seconds(seconds) (((usize)(seconds)) * 1000000000ull)
#endif

#ifndef m_milliseconds
#define m_milliseconds(milliseconds) (((usize)(milliseconds)) * 1000000ull)
#endif

#ifndef m_microseconds
#define m_microseconds(microseconds) ((usize)(microseconds) * 1000ull)
#endif

usize now_ns();

#endif //UTILS_H
