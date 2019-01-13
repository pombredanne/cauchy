#include "vm.h"
int main()
{
    // const char a[8] = "ABCDEFGH";
    // const long long a = 0x4847464544434241;
    __vm_retval("ABCDEFGH",8);
    __vm_exit(0);
    return 0;
}