riscv64-unknown-elf-gcc -I ./tinycrypt/lib/include/ ./tinycrypt/lib/source/sha256.c ./tinycrypt/lib/source/utils.c ./tinycrypt/lib/source/ecc.c ./tinycrypt/lib/source/ecc_dsa.c ./tinycrypt/lib/source/ecc_dh.c ecdsa.c -o ecdsa -s -nostdlib