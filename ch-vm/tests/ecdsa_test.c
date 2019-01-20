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
        break;
    }
    return retval;
}