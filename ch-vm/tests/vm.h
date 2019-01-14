#define __vm_retval(addr, size) __asm__ volatile( \
    "mv a5, %1\n\t"                               \
    "mv a6, %0\n\t"                               \
    "li a7, 0xCBFF\n\t"                           \
    "ecall\n\t"                                   \
    : /* no outputs */                            \
    : "r"(addr), "r"(size)                        \
    : "a5", "a6", "a7")

#define __vm_exit(ret) __asm__ volatile( \
    "li a0, %0\n\t"                      \
    "li a7, 93\n\t"                      \
    "ecall\n\t"                          \
    : /* no outputs */                   \
    : "g"(ret)                           \
    : /* no clobbers */)

#define __vm_call(sendbuff, sendsize, recvbuff, recvsize) __asm__ volatile( \
    "mv a3, %0\n\t"                                                         \
    "li a4, 8\n\t"\ 
        "mv a6, %1\n\t"\    
        "li a7, 0xCBFE\n\t"                                                 \
        "ecall\n\t"                                                         \
    : /* no output */                                                       \
    : "r"(recvbuff), "r"(sendbuff)                                          \
    : "a3", "a6", "a7")

// void __inline__ __vm_call(void *const sendbuff, const int send_size, void *const recvbuff, const int recvsize )
// {

// }