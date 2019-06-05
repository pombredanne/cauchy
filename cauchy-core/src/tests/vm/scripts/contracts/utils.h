#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

void *memset(void *dst, int c, size_t n);

// a_out = a_out + b
bool safeadd_u64(uint64_t *const a_out, uint64_t b);

// a_out = a_out - b
bool safesub_u64(uint64_t *const a_out, uint64_t b);