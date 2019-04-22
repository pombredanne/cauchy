#include "../vm.h"
#include "utils.h"
#include "stdint.h"

#define TXID_SZ     (32)
#define MAX_TOKENS  (uint32_t)(1000000000)

/*
First, it is necessary to study the facts, to multiply the number of observations, 
and then later to search for formulas that connect them so as thus to discern the 
particular laws governing a certain class of phenomena. In general, it is not until 
after these particular laws have been established that one can expect to discover 
and articulate the more general laws that complete theories by bringing a multitude 
of apparently very diverse phenomena together under a single governing principle.
*/

#define STORE_ADDR  ("46c55cb72c1363d2b908fd30c682c4291a00a8a582d62060112de14ae2bf5c03")

// Tinycrypt RNG implementation
int default_CSPRNG(uint8_t *dest, unsigned int size)
{
    __vm_rand(dest, size);
    return 1;
}

void main()
{
    uint8_t txid_sender[TXID_SZ];   // Buffer to hold the sender's TXID
    uint8_t msg[256];               // Buffer to hold the msg
    int txid_sz = 0;
    int msg_sz = 0;
    

    // First check if we have any messages waiting for us
    if( __vm_recv(txid_sender, &txid_sz, msg, &msg_sz))
    {

    }
    else
    {
        // We were woken up without a sender, that means we're starting new!
        // Welcome.jpg

        // The aux data should contain the seed address
        const uint8_t seed_addr[32];
        const uint32_t seed_amount = 0;

        // The first 32 bytes are the seed address
        __vm_auxdata((void*)seed_addr, 0, 32);
        // The next 4 bytes are the seed amount
        __vm_auxdata((void*)&seed_amount, 33, 4);

        // Set the balance of the seed address to the seed amount
        __vm_store(seed_addr, 32, &seed_amount, 4);

        // The amount in the bank is the max-seed
        const uint32_t num_bank_tokens = MAX_TOKENS-seed_amount;

        // The address for the "bank" of tokens is 64, making it inaccesible
        // from 32-bit addresses.  So even if a key for an address was found,
        // it could not access these tokens.
        __vm_store(STORE_ADDR, 64, &num_bank_tokens, sizeof(uint32_t));
        
    }
    
    __vm_exit(0);
}