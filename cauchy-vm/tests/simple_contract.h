#include <stdint.h>

typedef struct
{
    uint8_t from[32];
    uint8_t to[32];
    uint64_t ammount;
} transfer_t;