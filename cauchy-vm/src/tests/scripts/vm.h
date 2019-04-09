#include "stdint.h"

void __vm_send(const char *const txid, uint32_t txid_sz, void *const buff, uint32_t size)
{
    __asm__ volatile(
        "mv a3, %0\n\t"
        "mv a4, %1\n\t"
        "mv a5, %2\n\t"
        "mv a6, %3\n\t"
        "li a7, 0xCBFF\n\t"
        "ecall\n\t"
        : /* no outputs */
        : "r"(txid), "r"(txid_sz), "r"(buff), "r"(size)
        : "a3", "a4", "a5", "a6", "a7");
}

void __vm_recv(char *const txid, uint32_t *const txid_sz, void *const buff, uint32_t *const size)
{
    __asm__ volatile(
        "mv a3, %2\n\t"
        "mv a4, %3\n\t"
        "mv a5, %4\n\t"
        "mv a6, %5\n\t"
        "li a7, 0xCBFE\n\t"
        "ecall\n\t"
        "mv %0, s1\n\t"
        "mv %1, s2\n\t"
        : "=r" (*txid_sz), "=r" (*size)
        : "r"(txid), "r"(txid_sz), "r"(buff), "r"(size)
        : "a3", "a4", "a5", "a6", "a7");
}

void __vm_exit(const int ret)
{
    __asm__ volatile(
        "mv a0, %0\n\t"
        "li a7, 93\n\t"
        "ecall\n\t"
        : /* no outputs */
        : "r"(ret)
        : "a0", "a7");
}

#if 0
void __vm_call(const char *txid, void *sendbuff, int sendsize, void *recvbuff, int recvsize)
{
    __asm__ volatile(
        "mv a2, %4\n\t"
        "mv a3, %0\n\t"
        "mv a4, %1\n\t"
        "mv a5, %2\n\t"
        "mv a6, %3\n\t"
        "li a7, 0xCBFE\n\t"
        "ecall\n\t"
        : /* no output */
        : "r"(recvbuff), "r"(recvsize), "r"(sendbuff), "r"(sendsize), "r"(txid)
        : "a2", "a3", "a4", "a5", "a6", "a7");
}

void __vm_wait_for_call(void *const recv_addr, int size)
{
    __asm__ volatile(
        "mv a5, %0\n\t"
        "mv a6, %1\n\t"
        "li a7, 0xCBFB\n\t"
        "ecall\n\t"
        "li a0, 123\n\t"
        "li a7, 93\n\t"
        "ecall\n\t"
        : /* no output */
        : "r"(recv_addr), "r"(size)
        : "a0", "a5", "a6", "a7");
}

void __vm_getrand(void *const addr, int size)
{
    __asm__ volatile(
        "mv a5, %1\n\t"
        "mv a6, %0\n\t"
        "li a7, 0xCBFD\n\t"
        "ecall\n\t"
        : /* no outputs */
        : "r"(addr), "r"(size)
        : "a5", "a6", "a7");
}

void __vm_gettime(int *time)
{
    __asm__ volatile(
        "mv a5, %0\n\t"
        "li a7, 0xCBFC\n\t"
        "ecall\n\t"
        : /* no outputs */
        : "r"(time)
        : "a5", "a7");
}
#endif 