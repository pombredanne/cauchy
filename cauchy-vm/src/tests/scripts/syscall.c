#include "vm.h"

_start()
{
    __vm_retbytes("DEADBEEF", 8);
    __vm_exit(0);
}