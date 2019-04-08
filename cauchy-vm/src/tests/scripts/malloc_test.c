#include "vm.h"
#include <sys/types.h>
#include <stdlib.h>

#define BRK_MIN 0x00800000
#define BRK_MAX 0x07800000

void*
_sbrk(ptrdiff_t incr)
{
  static uintptr_t p = BRK_MIN;
  uintptr_t start = p;
  p += incr;
  if (p > BRK_MAX) {
    return (void *) (-1);
  }
  return start;
}

void main(void)
{
    char sender_txid[128];
    char data[128];
    int sender_addr_size = 0;
    int data_size = 0;
    int *ptr = (int*) malloc(sizeof(int));

    __vm_recv(sender_txid, &sender_addr_size, data, &data_size);
    __vm_send(sender_txid, sender_addr_size, data, data_size);
    __vm_send("RECVR", 5, "DEADBEEF is happyBEEF", 21);
    __vm_exit(0);
}