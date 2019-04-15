#include "vm.h"


void *memset(void *dst, int c, size_t n)
{
    if (n) {
        char *d = dst;

        do {
            *d++ = c;
        } while (--n);
    }
    return dst;
}

void _start()
{
    char sender_txid[128];
    char data[128];
    char buff[128] = {'\0'};
    int sender_addr_size = 0;
    int data_size = 0;
    __vm_recv(sender_txid, &sender_addr_size, data, &data_size);
    __vm_send(sender_txid, sender_addr_size, data, data_size);
    __vm_store("TestKey", 7, "TestVal", 7);
    __vm_lookup("TestKey", 7, buff, 7);
    __vm_send("TestKey", 7, buff, 7 );
    __vm_send("RECVR", 5, "DEADBEEF is happyBEEF", 21);
    __vm_exit(0);
}