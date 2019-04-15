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
    char sender_txid[32];
    char data[32];
    char buff[32];
    int sender_addr_size = 0;
    int data_size = 0;

    memset(sender_txid, 'A', sizeof(sender_txid));
    memset(data, 'B', sizeof(data));
    memset(buff, 'C', sizeof(buff));

    __vm_send("TestBasicSend", 13, "DEADBEEF is happyBEEF", 21);
    
    __vm_recv(sender_txid, &sender_addr_size, data, &data_size);
    __vm_send(sender_txid, sender_addr_size + 10, data, data_size + 10);
    
    __vm_store("TestKey", 7, "TestVal", 7);
    __vm_lookup("TestKey", 7, buff, 7);
    __vm_send("TestKey", 7, buff, 7 + 10 );

    memset(buff, 'D', sizeof(buff));
    __vm_auxdata(buff, &data_size);
    __vm_send("TestAux", 7, buff, data_size + 10);

    __vm_sendfromaux(5, 10);

    __vm_exit(0);
}