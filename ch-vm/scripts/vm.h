// Implement _start()
// extern int main();
// void _start()
// {
//     main();
// }

#define __vm_retbytes(addr, size) __asm__ volatile( \
    "mv a5, %1\n\t"                                 \
    "mv a6, %0\n\t"                                 \
    "li a7, 0xCBFF\n\t"                             \
    "ecall\n\t"                                     \
    : /* no outputs */                              \
    : "r"(addr), "r"(size)                          \
    : "a5", "a6", "a7")

#define __vm_exit(ret) __asm__ volatile( \
    "li a0, %0\n\t"                      \
    "li a7, 93\n\t"                      \
    "ecall\n\t"                          \
    : /* no outputs */                   \
    : "g"(ret)                           \
    : "a0", "a7")

#define __vm_call(txid, sendbuff, sendsize, recvbuff, recvsize) __asm__ volatile( \
    "mv a2, %4\n\t"                                                               \
    "mv a3, %0\n\t"                                                               \
    "mv a4, %1\n\t"                                                               \
    "mv a5, %2\n\t"                                                               \
    "mv a6, %3\n\t"                                                               \
    "li a7, 0xCBFE\n\t"                                                           \
    "ecall\n\t"                                                                   \
    : /* no output */                                                             \
    : "r"(recvbuff), "r"(recvsize), "r"(sendbuff), "r"(sendsize), "r"(txid)       \
    : "a2", "a3", "a4", "a5", "a6", "a7")

#define __vm_getrand(addr, size) __asm__ volatile( \
    "mv a5, %1\n\t"                                 \
    "mv a6, %0\n\t"                                 \
    "li a7, 0xCBFD\n\t"                             \
    "ecall\n\t"                                     \
    : /* no outputs */                              \
    : "r"(addr), "r"(size)                          \
    : "a5", "a6", "a7")



// typedef unsigned int size_t;

// void *memset(void *dst, int c, size_t n)
// {
//     if (n)
//     {
//         char *d = dst;

//         do
//         {
//             *d++ = c;
//         } while (--n);
//     }
//     return dst;
// }

// void *memcpy(void *dest, const void *src, size_t len)
// {
//     char *d = dest;
//     const char *s = src;
//     while (len--)
//         *d++ = *s++;
//     return dest;
// }

// int rand(void)
// {
//     return 0xDEADBEEF;
// }

// void __inline__ __vm_call(void *const sendbuff, const int send_size, void *const recvbuff, const int recvsize )
// {

// }