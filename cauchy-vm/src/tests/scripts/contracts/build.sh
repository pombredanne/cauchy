TINYCRYPT_DIR=../tinycrypt/lib/

riscv64-unknown-elf-gcc -I $TINYCRYPT_DIR/include/ $TINYCRYPT_DIR/source/sha256.c $TINYCRYPT_DIR/source/utils.c $TINYCRYPT_DIR/source/ecc.c $TINYCRYPT_DIR/source/ecc_dsa.c $TINYCRYPT_DIR/source/ecc_dh.c gas.c utils.c -o gas -s -Os