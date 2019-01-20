#include "vm.h"

int main(int argc, char *argv[])
{
    int time = 0;
    __vm_gettime(&time);
    __vm_retbytes(&time, 4);
    // valid up to 1600000000 => Sunday, September 13, 2020 12:26:40 PM GMT
    if ( time > 1547962054 && time < 1600000000)
        return 1;
    else
        return 0;
}