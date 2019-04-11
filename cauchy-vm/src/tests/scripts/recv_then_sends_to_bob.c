#include "vm.h"

void _start()
{
    char sender_txid[128];
    char data[128];
    int sender_addr_size = 0;
    int data_size = 0;
    if(__vm_recv(sender_txid, &sender_addr_size, data, &data_size))
    {
        __vm_send("BOB", 3, data, data_size);
    }
    else{
        __vm_send("BOB", 3, "Sorry BOB, I didn't receive anything to send you :-(", 52);
    }
    __vm_exit(0);
}