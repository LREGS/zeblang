#!/bin/bash
echo compiling $1
target/debug/zeblang $1
nasm -felf64 ${1%.zb}.asm
ld ${1%.zb}.o -o ${1%.zb}

#cleanup
rm ${1%.zb}.asm
rm ${1%.zb}.o