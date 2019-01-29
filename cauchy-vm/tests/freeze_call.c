#include "vm.h"
#include <string.h>


int main(int argc, char *argv[])
// int main()
{
    const char a[8] = "hello";
    // memcpy(a, argv[1], 5);
    // __vm_retval("ABCDEFGH",8);
    char recv_buff[32];
    
    // This is a dummy example of freeze's pretend txid
    __vm_call("12DD9774CC96E18B5B1B5D4A4B1E0724C4B3E3F37A3EED1AF6A1DAC7CFDBC3E3", argv[1], *(int *)argv[2], recv_buff, 32);
    __vm_retbytes(recv_buff, 32);
    // __vm_exit(0);
    return 0;
}