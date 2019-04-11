#include "vm.h"

void _start()
{
    __vm_send("ALICE", 5, "Hello ALICE", 11);
    __vm_exit(0);
}