#define __vm_retval(addr, size) __asm__ volatile( \
    "mv a5, %1\n\t"                               \
    "mv a6, %0\n\t"                               \
    "li a7, 0xCBFF\n\t"                           \
    "ecall\n\t"                                   \
    : /* no outputs */                            \
    : "r"(addr), "r"(size)                        \
    : "a5", "a6", "a7")

// #define __vm_call(addrin, insize, addrout, outsize) __asm__( \
//     "ld a3, %1\n\t"                                          \
//     "mv a4, %0\n\t"                                          \
//     "mv a5, %3\n\t"                                          \
//     "mv a6, %2\n\t"                                          \
//     "li a7, 0xCBFE\n\t"                                      \
//     "ecall\n\t"                                              \
//     : "=m"(addrin), "=m"(insize)                             \
//     : "r"(addrout), "r"(outsize)                             \
//     : "a3", "a4", "a5", "a6", "a7")

#define __vm_exit(ret) __asm__ volatile( \
    "li a0, %0\n\t"                      \
    "li a7, 93\n\t"                      \
    "ecall\n\t"                          \
    : /* no outputs */                   \
    : "g"(ret)                           \
    : /* no clobbers */)