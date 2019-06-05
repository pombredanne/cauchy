#include "stdint.h"

// Type of message
typedef enum
{
    E_MSG_NONE,
    E_MSG_TRANSFER,
    E_MSG_BALANCE,
    E_MSG_ERR=0xFFFF
} msgType;

typedef struct
{
    msgType type;
    uint8_t data[32];
} gasMsg;
