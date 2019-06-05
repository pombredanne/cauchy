#include "../vm.h"
#include "utils.h"
#include "stdint.h"
#include "string.h"
#include "gas_msg.h"
#include <tinycrypt/sha256.h>
#include <tinycrypt/ecc.h>
#include <tinycrypt/ecc_dh.h>
#include <tinycrypt/ecc_dsa.h>

#define DEBUG (1)

#if (DEBUG==1)
#include "stdio.h"
#endif

#define MAX_TOKENS ((uint64_t)1000000000 * (uint64_t)1000000000)
#define TXID_SZ (32)
#define ADDR_SZ (32)
#define PUBKEY_SZ (64)
#define SIG_SZ (64)

/*
    First, it is necessary to study the facts, to multiply the number of observations, 
    and then later to search for formulas that connect them so as thus to discern the 
    particular laws governing a certain class of phenomena. In general, it is not until 
    after these particular laws have been established that one can expect to discover 
    and articulate the more general laws that complete theories by bringing a multitude 
    of apparently very diverse phenomena together under a single governing principle.
*/

#define STORE_ADDR ("46c55cb72c1363d2b908fd30c682c4291a00a8a582d62060112de14ae2bf5c03")

// Tinycrypt RNG implementation
int default_CSPRNG(uint8_t *dest, unsigned int size)
{
    __vm_rand(dest, size);
    return 1;
}

void main()
{
    uint8_t txid_sender[TXID_SZ]; // Buffer to hold the sender's TXID
    uint8_t msg[1024] = {0};
    int txid_sz = TXID_SZ;
    int msg_sz = sizeof(gasMsg);

    // First check if we have any messages waiting for us
    // if (!)
    __vm_recv(txid_sender, &txid_sz, &msg, &msg_sz);
    // TODO: Remove testing if
    const uint8_t seed_addr[32];
    __vm_auxdata((void *)seed_addr, 0, 32);
    if (__vm_lookup(seed_addr, ADDR_SZ, NULL, 0) != 0)
    {
        // We were woken up without a sender, that means we're starting new!
        // Welcome.jpg

        // The aux data contains the seed address and amount
        // const uint8_t seed_addr[32];
        uint64_t seed_amount = 0;

        // The first 32 bytes are the seed address
        __vm_auxdata((void *)seed_addr, 0, 32);
        // // The next 4 bytes are the seed amount
        __vm_auxdata((void *)&seed_amount, 32, sizeof(seed_amount));

        // The amount in the bank is the max-seed
        uint64_t num_bank_tokens = MAX_TOKENS;

        if (safesub_u64(&num_bank_tokens, seed_amount))
        {
            // Set the balance of the seed address to the seed amount
            __vm_store(seed_addr, 32, &seed_amount, sizeof(seed_amount));

            // The address for the "bank" of tokens is 64, making it inaccesible
            // from 32-bit addresses.  So even if a key for an address was found,
            // it could not access these tokens.
            __vm_store(STORE_ADDR, 64, &num_bank_tokens, sizeof(num_bank_tokens));
        }
    } // endif recv_message()
    else
    {
        // TODO: Parse messages, check signatures
        const msgType msg_type = (msgType)((((short)msg[1]) << 8) + (short)msg[0]);
        switch (msg_type)
        {
        case E_MSG_BALANCE:
        {
            // A simple balance request.  The data is the address to look up
            uint32_t balance = 0;
            const int retval = __vm_lookup(msg + 2, 32, &balance, sizeof(balance));
            if (retval == 0)
            {
                uint8_t buff[sizeof(msgType) + sizeof(balance)];
                memcpy(buff, &msg_type, sizeof(msgType));
                memcpy(buff + sizeof(msgType), &balance, sizeof(balance));
                __vm_send(txid_sender, TXID_SZ, (void *const)buff, sizeof(msgType) + sizeof(balance));
            }
            else
            {
                __vm_send(txid_sender, TXID_SZ, (void *const) "\xff\xff"
                                                              "Account not found",
                          2 + 17);
            }

            break;
        }
        case E_MSG_TRANSFER:
        {
            // A transfer from one account to another
            uint64_t amount = 0;
            const unsigned int expected_size = sizeof(msg_type) + ADDR_SZ + PUBKEY_SZ + sizeof(amount) + SIG_SZ;
            if (msg_sz == expected_size)
            {
                uint8_t pubkey_from[PUBKEY_SZ + 1] = {0};
                uint8_t acct_from[ADDR_SZ + 1] = {0};
                uint8_t acct_to[ADDR_SZ + 1] = {0};
                uint8_t sig[SIG_SZ];
                uint8_t hash[32] = {0};
                struct tc_sha256_state_struct s;
                uint64_t acct_from_balance = 0;
                uint64_t acct_to_balance = 0;

                // Parse the message into its parts
                // || msg (2) || addr_to (ADDR_SZ) || pubkey_from (PUBKEY_SZ) || amount (8) || sig (SIG_SZ)
                memcpy(acct_to, msg + 2, ADDR_SZ);
                memcpy(pubkey_from, msg + 2 + ADDR_SZ, PUBKEY_SZ);
                memcpy(&amount, msg + 2 + ADDR_SZ + PUBKEY_SZ, sizeof(amount));
                memcpy(sig, msg + 2 + ADDR_SZ + PUBKEY_SZ + sizeof(amount), SIG_SZ);

                // Generate the acct_from and check the balance
                tc_sha256_init(&s);
                tc_sha256_update(&s, pubkey_from, PUBKEY_SZ);
                tc_sha256_final(acct_from, &s);
                if (__vm_lookup(acct_from, ADDR_SZ, &acct_from_balance, sizeof(acct_from_balance)) == 0)
                {
                    // If the sender has the funds...
                    if (acct_from_balance >= amount)
                    {
                        // Hash the message and verify the signature
                        tc_sha256_init(&s);
                        tc_sha256_update(&s, msg, expected_size - SIG_SZ);
                        tc_sha256_final(hash, &s);
                        if (uECC_verify(pubkey_from, hash, 32, sig, uECC_secp256r1()))
                        {
                            // Update the recipient's current balance
                            __vm_lookup(acct_to, ADDR_SZ, &acct_to_balance, sizeof(acct_from_balance));

                            // Signature is good!  Is there an overflow condition?
                            if (safesub_u64(&acct_from_balance, amount) && safeadd_u64(&acct_to_balance, amount))
                            {
                                __vm_store(acct_from, 32, &acct_from_balance, sizeof(acct_from_balance));
                                __vm_store(acct_to, 32, &acct_to_balance, sizeof(acct_from_balance));
#if (DEBUG == 1)
                                char strbuff[255] = {0};
                                acct_to[ADDR_SZ] = '\0';
                                const int len = snprintf(strbuff, 255, "You sent %u/%u gas from %s to %s.  Their balance is now %u", amount, acct_from_balance + amount, pubkey_from, acct_to, acct_to_balance);
                                __vm_send(txid_sender, TXID_SZ, (void *const)strbuff, len);
#endif
                            } // endif safemath()
                            else
                            {
                                // Error condition
                            }
                        } // endif sig_verify()
                        else
                        {
                            __vm_send(txid_sender, TXID_SZ, (void *const) "Sig failed", 10);
                        }
                    } // endif (acct_from_balance >= amount)
                    else
                    {
#if (DEBUG == 1)
                        char strbuff[255] = {0};
                        snprintf(strbuff, 255, "Insufficient balance (%u / %u)!!!", amount, acct_from_balance);
                        __vm_send(txid_sender, TXID_SZ, (void *const)strbuff, 255);
#endif
                    }
                } // endif from_addr exists()
                else
                {
                    #if (DEBUG==1)
                    char strbuff[255] = {0};
                    snprintf(strbuff, 255, "Account %s does not exist!", acct_from);
                    __vm_send(txid_sender, TXID_SZ, (void *const)strbuff, 255);
                    #endif
                }
            } // endif msg == expected_size
            else
            {
                // Do nothing, something is malformed
            }

            break;
        } // endcase E_MSG_TRANSFER:
        default:
        {
            const gasMsg response = {.type = E_MSG_ERR, .data = "Bad Message"};
            __vm_send(txid_sender, TXID_SZ, (void *const) & response, 34);
            break;
        }
        } // endswitch (msg_type)
    }

    __vm_exit(0);
}