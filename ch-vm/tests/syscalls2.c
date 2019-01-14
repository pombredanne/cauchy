#include "vm.h"
int main()
{
    const char a[8] = "ABCDEFGH";
    // __vm_retval("ABCDEFGH",8);
    char buff[8] = {1, 2, 3, 4, 5, 6, 7, 8};
    __vm_call(a, 8, buff, 8);
    
    __vm_retval(buff, 8);
    __vm_exit(0);
    return 0;
}