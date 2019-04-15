#include "vm.h"
#include "string.h"

void *memset(void *dst, int c, size_t n)
{
    if (n) {
        char *d = dst;

        do {
            *d++ = c;
        } while (--n);
    }
    return dst;
}

int strcmp (const char *p1, const char *p2)
{
  const unsigned char *s1 = (const unsigned char *) p1;
  const unsigned char *s2 = (const unsigned char *) p2;
  unsigned char c1, c2;
  do
    {
      c1 = (unsigned char) *s1++;
      c2 = (unsigned char) *s2++;
      if (c1 == '\0')
        return c1 - c2;
    }
  while (c1 == c2);
  return c1 - c2;
}

void _start()
{
    char buff[128] = {'\0'};
    __vm_store("TestKey", 7, "TestVal", 7);
    __vm_lookup("TestKey", 7, buff, 7);
    uint8_t retval = strcmp(buff, "TestVal");
    __vm_exit(retval);
}