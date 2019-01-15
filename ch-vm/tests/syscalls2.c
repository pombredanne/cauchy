#include "vm.h"
#include <string.h>

// int atoi(const char *const str)
// {
// 	int res = 0; // Initialize result

// 	// Iterate through all characters of input string and
// 	// update result
// 	for (int i = 0; str[i] != '\0'; ++i)
// 		res = res * 10 + str[i] - '0';

// 	// return result.
// 	return res;
// }

int main(int argc, char *argv[])
// int main()
{
    const char a[8] = "hello";
    // memcpy(a, argv[1], 5);
    // __vm_retval("ABCDEFGH",8);
    char recv_buff[32];
    
    __vm_call(argv[1], 5, recv_buff, 32);
    __vm_retbytes(recv_buff, 32);
    __vm_exit(0);
    // return 0;
}