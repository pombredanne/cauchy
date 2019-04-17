#include "vm.h"
#include <tinycrypt/sha256.h>
#include <tinycrypt/ecc.h>
#include <tinycrypt/ecc_dh.h>
#include <tinycrypt/ecc_dsa.h>

void *memset(void *dst, int c, size_t n)
{
    if (n)
    {
        char *d = dst;

        do
        {
            *d++ = c;
        } while (--n);
    }
    return dst;
}

void *memcpy(void *dst, const void *src, size_t len)
{
    size_t i;

    /*
         * memcpy does not support overlapping buffers, so always do it
         * forwards. (Don't change this without adjusting memmove.)
         *
         * For speedy copying, optimize the common case where both pointers
         * and the length are word-aligned, and copy word-at-a-time instead
         * of byte-at-a-time. Otherwise, copy by bytes.
         *
         * The alignment logic below should be portable. We rely on
         * the compiler to be reasonably intelligent about optimizing
         * the divides and modulos out. Fortunately, it is.
         */

    if ((uintptr_t)dst % sizeof(long) == 0 &&
        (uintptr_t)src % sizeof(long) == 0 &&
        len % sizeof(long) == 0)
    {

        long *d = dst;
        const long *s = src;

        for (i = 0; i < len / sizeof(long); i++)
        {
            d[i] = s[i];
        }
    }
    else
    {
        char *d = dst;
        const char *s = src;

        for (i = 0; i < len; i++)
        {
            d[i] = s[i];
        }
    }

    return dst;
}

int default_CSPRNG(uint8_t *dest, unsigned int size)
{
    __vm_rand(dest, size);
    return 1;
}

void _start()
{
    uint8_t priv[32];
    uint8_t pubkey[64];
    uint8_t msg[256];
    uint8_t sig[64];
    int msg_size = 256;

    const struct uECC_Curve_t * curve = uECC_secp256r1();

    uECC_make_key(pubkey, priv, curve);
    __vm_send("PrivKey", 7, priv, 32);
    __vm_send("PubKey", 6, pubkey, 64);

    __vm_auxdata(msg, &msg_size);

    // uECC_sign(priv, msg, 32, sig, curve);
    __vm_send("Sig", 3, sig, 64);
    __vm_exit(0);
}