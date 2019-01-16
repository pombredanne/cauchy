#include "tinycrypt/ecc.h"
#include <tinycrypt/ecc_dsa.h>
#include <tinycrypt/ecc_dh.h>
#include <stdlib.h>
#include <string.h>
#include "vm.h"

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
    char privkey[32] = {0};
    char pubkey[64] = {0};
    char sig[64] = {0};
    char hash[32] = {0};

    memcpy(pubkey, argv[1], sizeof(pubkey));
    memcpy(sig, argv[1]+sizeof(pubkey), sizeof(sig));
    memcpy(hash, argv[1]+sizeof(pubkey)+sizeof(sig), sizeof(hash));

    uECC_Curve curve = uECC_secp256r1();

    // if (!uECC_make_key(pubkey, privkey, curve))
    // {
    //     return 1;
    // }

    // if (!uECC_sign(privkey, hash, sizeof(hash), sig, curve))
    // {
    //     return 1;
    // }

    if (!uECC_verify(pubkey, hash, sizeof(hash), sig, curve))
    {
        return 1;
    }

    // default_CSPRNG(pubkey, 64);
    __vm_retbytes(sig, sizeof(sig));
    // __vm_exit(0);
    return 0;
}