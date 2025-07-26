#ifndef UTILS_H
#define UTILS_H

#include <assert.h>

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
    do { assert((condition) && ("in " __FILE__ ": " message)); } while (0)

#define m_dev_assert_null(pointer, operation) \
    m_dev_assert(((pointer) != 0), ("in " __FILE__ ": can't perform " operation " on null"))

#else

#define m_dev_assert(condition, message) ((void*)0)
#define m_dev_assert(pointer, operation) ((void*)0)

#endif

#endif //UTILS_H
