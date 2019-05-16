#include "vm.h"
#include <tinycrypt/sha256.h>

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

void _start()
{
    uint8_t digest[32];
    uint8_t msg[] = "Test message";
    struct tc_sha256_state_struct s;
    tc_sha256_init(&s);
    tc_sha256_update(&s, msg, 12);
    tc_sha256_final(digest, &s);
    
    __vm_send("sha256Test", 10, digest, 32);
    __vm_exit(0);
}