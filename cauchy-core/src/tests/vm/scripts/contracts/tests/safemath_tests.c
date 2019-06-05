#include "../utils.h"

int main(void)
{
    bool is_ok = true;
    uint64_t a = 0;
    
    // Normal addition
    is_ok &= safeadd_u64(&a, 100);          // a = 100
    is_ok &= (a == 100);
    
    // Normal subtraction
    is_ok &= safesub_u64(&a, 100);          // a = 0
    is_ok &= (a == 0);

    // Underflow subtraction should fail
    is_ok &= !(safesub_u64(&a, 1));         // a = 0
    is_ok &= (a == 0);

    // Normal addition to max value
    is_ok &= safeadd_u64(&a, UINT64_MAX);   // a = UINT64_MAX
    is_ok &= (a == UINT64_MAX);

    // Overflow addition should fail
    is_ok &= !(safeadd_u64(&a, 1));         // a = UINT64_MAX
    is_ok &= (a == UINT64_MAX);

    return (is_ok && (a == UINT64_MAX))? 100 : 0;
}