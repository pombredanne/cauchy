#include "vm.h"
#include <stdint.h>

int main(int argc, char *argv[])
// int main()
{
    // Call simple_contract script with account index and an increment
    uint8_t recv_buff[8] = {0};

    uint8_t data[2] = {0x0, 0x1};
    __vm_call("0ECE6BA565D32F43A5A2E5AED2E39F0359084A88A99D1CF1BBE91E6F4315D0DF", data, 2, recv_buff, 8);
    __vm_retbytes(recv_buff, 8);
    
    return 0;
}