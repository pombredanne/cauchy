#include "vm.h"
#include "string.h"

int main(int argc, char *argv[])
{
    char buff[8] = {0};

    __vm_wait_for_call(buff, 8);

    if(memcmp(buff, "DEADBEEF", 8))
        return 2;

    return 1;
}