#include "vm.h"

_start()
{
    __vm_reply("DEADBEEF is happyBEEF", 21);
    __vm_exit(0);
}