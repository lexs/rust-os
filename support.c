#include <stddef.h> /* size_t */
void *memcpy(void *dest, const void *src, size_t n)
{
    char *dp = dest;
    const char *sp = src;
    while (n--)
        *dp++ = *sp++;
    return dest;
}
/*
void *memmove(void *dest, const void *src, size_t n)
{
    unsigned char *pd = dest;
    const unsigned char *ps = src;
    if (__np_anyptrlt(ps, pd))
        for (pd += n, ps += n; n--;)
            *--pd = *--ps;
    else
        while(n--)
            *pd++ = *ps++;
    return dest;
}
*/
void *memset(void *s, int c, size_t n)
{
    unsigned char* p=s;
    while(n--)
        *p++ = (unsigned char)c;
    return s;
}