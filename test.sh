#!/bin/sh

rm test -f &> /dev/null

clang -o test libsafedrive/src/test.c -Ldist-x86_64-apple-darwin/dep/lib -Ldist-x86_64-apple-darwin/lib -Idist-x86_64-apple-darwin/include -lz -lsodium -lsafedrive -framework Foundation -framework Security -O2

time RUST_BACKTRACE=1 ./test

