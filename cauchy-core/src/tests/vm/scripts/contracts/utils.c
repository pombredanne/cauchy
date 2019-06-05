#include "utils.h"
#include <limits.h>
#include <stdbool.h>

// a_out = a_out + b
bool safeadd_u64(uint64_t *const a_out, uint64_t b)
{
    const bool is_safe = (a_out != NULL) && (__UINT64_MAX__ - *a_out >= b);
    if(is_safe)
        *a_out += b;
    return is_safe;
}

// a_out = a_out - b
bool safesub_u64(uint64_t *const a_out, uint64_t b)
{
    const bool is_safe =  (a_out != NULL) && (*a_out >= b);
    if(is_safe)
        *a_out -= b;
    return is_safe;
}