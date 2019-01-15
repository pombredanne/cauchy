#!/bin/sh

riscv64-unknown-elf-gcc -I"./" utils.c sha256.c -o sha256 -s