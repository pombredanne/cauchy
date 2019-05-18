#include "vm.h"

void _start()
{
    __vm_send("RECVR", 5, "DEADBEEF is happyBEEF", 21);
    __vm_exit(0);
}