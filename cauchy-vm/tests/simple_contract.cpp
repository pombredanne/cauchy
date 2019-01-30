#include "vm.h"
#include "stdint.h"
#include <cstring>
#include <map>
#include "simple_contract.h"
using namespace std;

int main(int argc, char *argv[])
{
    map<uint8_t, uint64_t> balances;

    // for(;;)
    // {
    //     uint8_t buff[2] = {0};

    //     // Program will block here and wait
    //     // __vm_wait_for_call(buff, 2);

    //     uint8_t account = 0;
    //     int8_t increment = 0;

    //     memcpy(&account, (uint8_t*)buff+0, 1);
    //     memcpy(&increment, (int8_t*)buff+1, 1);

    // balances.insert_or_assign(0, 0);

    //     balances[account]+= increment;

    //     // __vm_retbytes(&balances[account], sizeof(uint64_t));
    //     return 1;
    // }

    // int i = 0;
    // char *buffer;


    // buffer = (char *)malloc(i + 1);
    // if (buffer == NULL)
    //     exit(1);

    // free(buffer);

    return 1;
}