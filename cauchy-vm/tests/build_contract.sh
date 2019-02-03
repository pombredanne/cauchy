#!/bin/sh

riscv64-unknown-elf-gcc simple_contract_call.c -o simple_contract_call -s
riscv64-unknown-elf-gcc simple_contract.c -o simple_contract -s
cp simple_contract ../scripts/0ECE6BA565D32F43A5A2E5AED2E39F0359084A88A99D1CF1BBE91E6F4315D0DF