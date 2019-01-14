#include "vm.h"
int main()
{
    const char a[8] = "ABCDEFGH";
    // const long long a = 0x4847464544434241;
    // __vm_retval("ABCDEFGH",8);
    char buff[8] = {1, 2, 3, 4, 5, 6, 7, 8};
    // __vm_call(buff, 8, a, 8);
    __asm__ volatile(
        "ld a3, %0\n\t"
        "li a4, 8\n\t"
        "mv a6, %1\n\t"
        "li a7, 0xCBFE\n\t"
        "ecall\n\t"
        : "=m"(buff)
        : "r"(a)
        : "a3", "a6", "a7");
    __vm_retval(buff, 8);
    __vm_exit(0);
    return 0;
}