#include "vm.h"

void _start()
{
    char sender_txid[128];
    char data[128];
    int sender_addr_size = 0;
    int data_size = 0;
    __vm_recv(sender_txid, &sender_addr_size, data, &data_size);
    __vm_send(sender_txid, sender_addr_size, data, data_size);
    __vm_send("RECVR", 5, "DEADBEEF is happyBEEF", 21);
    __vm_exit(0);
}