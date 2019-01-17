#include "tinycrypt/ecc.h"
#include <tinycrypt/ecc_dsa.h>
#include <tinycrypt/ecc_dh.h>
#include <stdlib.h>
#include <string.h>
#include "vm.h"
#include "vm_utils.h"

int default_CSPRNG(uint8_t *dest, unsigned int size)
{
    int bytes_remaining = size;
    int shift = 0;
    int randint = rand();

    while (bytes_remaining-- > 0)
    {
        if (bytes_remaining % sizeof(int) == 0)
        {
            randint = rand();
            shift = 0;
        }
        dest[bytes_remaining - 1] = randint << shift++;
    }
    return 1;
}

int main(int argc, char *argv[])
{
    int retval = 0;
    char privkey[32] = {0};
    char pubkey[64] = {0};
    char sig[64] = {0};
    char hash[32] = {0};

    memcpy(pubkey, argv[1], sizeof(pubkey));
    memcpy(sig, argv[1] + sizeof(pubkey), sizeof(sig));
    memcpy(hash, argv[1] + sizeof(pubkey) + sizeof(sig), sizeof(hash));
    // hex2bin("DEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF", hash, 64, NULL);

    uECC_Curve curve = uECC_secp256r1();

    const int func = *argv[0] & 0xFF;
    switch (func)
    {
    case 0:
        if (uECC_make_key(pubkey, privkey, curve))
        {
            __vm_retbytes(pubkey, sizeof(pubkey));
            retval = 1;
        }
        break;

    case 1:
        if (!uECC_make_key(pubkey, privkey, curve))
        {
            return 0;
        }

        if (uECC_sign(privkey, hash, sizeof(hash), sig, curve))
        {
            __vm_retbytes(sig, sizeof(sig));
            retval = 1;
        }
        break;
    case 2:
        if (uECC_verify(pubkey, hash, sizeof(hash), sig, curve))
        {
            retval = 1;
        }

        break;
    default:
         __vm_retbytes(argv[0], 1);
        break;
    }

    // __asm__ volatile(
    //     "mv a3, %0\n\t"
    //     "mv a4, %1\n\t"
    //     "mv a5, %2\n\t"
    //     "mv a6, %3\n\t"
    //     "li a7, 0xCBFE\n\t"
    //     "ecall\n\t"
    //     : /* no output */
    //     : "r"(recvbuff), "r"(recvsize), "r"(sendbuff), "r"(sendsize)
    //     : "a3", "a4", "a5", "a6", "a7");

    // default_CSPRNG(pubkey, 64);
    // __vm_retbytes(hash, sizeof(hash));
    // __vm_exit(retval);
    return retval;
}