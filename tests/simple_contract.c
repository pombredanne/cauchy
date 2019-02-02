#include "vm.h"
#include "string.h"
#include "stdint.h"

int main(int argc, char *argv[])
{
    uint8_t balances[8] = {0};
    for(;;)
    {
        uint8_t buff[2] = {0};
        
        // Program will block here and wait
        __vm_wait_for_call(buff, 2);

        uint8_t account = 0;
        int8_t increment = 0;

        memcpy(&account, (uint8_t*)buff+0, 1);
        memcpy(&increment, (int8_t*)buff+1, 1);

        // balances[account] += increment;
        balances[account]+= increment;

        __vm_retbytes(balances, 8);
    }
    
    return 1;
}