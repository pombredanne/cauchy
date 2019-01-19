#include "tinycrypt/ecc.h"
#include <tinycrypt/ecc_dsa.h>
#include <tinycrypt/ecc_dh.h>
#include <stdlib.h>
#include <string.h>
#include "vm.h"
#include "vm_utils.h"

int default_CSPRNG(uint8_t *dest, unsigned int size)
{
    __vm_getrand(dest, size);
    return 1;
}

int main(int argc, char *argv[])
{
    int retval = 0;
    char privkey[32] = {1};
    char hash[32] = {2};
    char pubkey[64] = {3};
    char sig[64] = {4};
    // hex2bin("DEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEFDEADBEEF", hash, 64, NULL);

    uECC_Curve curve = uECC_secp256r1();

    const int func = *argv[0] & 0xFF;
    /*
        0 = make keypair (insecure, testing only)
        1 = sign hash
        2 = verify sig
    */
    switch (func)
    {
    case 0:
        if (uECC_make_key(pubkey, privkey, curve))
        {
            // Copy both to buff before copying out
            char buff[32 + 64] = {0};
            memcpy(buff, privkey, 32);
            memcpy(buff + 32, pubkey, 64);

            // Return privkey || pubkey
            __vm_retbytes(buff, sizeof(buff));
            retval = 0;
        }
        break;

    case 1:
    {
        memcpy(privkey, argv[1], sizeof(privkey));
        memcpy(hash, argv[1] + sizeof(privkey), sizeof(hash));

        if (uECC_sign(privkey, hash, sizeof(hash), sig, curve))
        {
            __vm_retbytes(sig, sizeof(sig));
            retval = 1;
        }
        break;
    }
    case 2:
        memcpy(pubkey, argv[1], sizeof(pubkey));
        memcpy(sig, argv[1] + sizeof(pubkey), sizeof(sig));
        memcpy(hash, argv[1] + sizeof(pubkey) + sizeof(sig), sizeof(hash));

        __vm_retbytes(sig, sizeof(sig));
        if (uECC_verify(pubkey, hash, sizeof(hash), sig, curve))
        {
            retval = 2;
        }

        break;
    default:
        // __vm_retbytes(argv[0], 1);
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