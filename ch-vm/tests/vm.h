#define __vm_retval(addr, size) __asm__ volatile ( \
    "mv a5, %1\n\t"                      \
    "mv a6, %0\n\t"                      \
    "li a7, 0xCBFF\n\t"                  \
    "ecall\n\t"                          \
    : /* no outputs */                   \
    : "r"(addr), "r"(size)               \
    : "a5", "a6", "a7")

#define __vm_exit(ret) __asm__ volatile ( \
    "li a0, %0\n\t"             \
    "li a7, 93\n\t"             \
    "ecall\n\t"                 \
    : /* no outputs */          \
    : "g"(ret)                  \
    : /* no clobbers */)