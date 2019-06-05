#!/bin/sh

echo -n "Safemath tests..."
riscv64-unknown-elf-gcc ../utils.c safemath_tests.c -o safemath_tests -s -Os
riscv64-unknown-elf-run safemath_tests
if [ $? -ne 100 ]
then
    echo "FAIL"
else
    echo "Pass"
fi