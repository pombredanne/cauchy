#include "vm.h"

int main(int argc, char *argv[])
{
    __asm__ volatile(
        "li a7, 0xCBFB\n\t"
        "ecall\n\t"
        "li a0, 0xFF\n\t"
        "li a7, 93\n\t"
        "ecall\n\t"
        :
        :
        :
    );

    return 1;
}