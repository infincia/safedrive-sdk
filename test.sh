#!/usr/bin/env bash

set -e

rm test -f &> /dev/null

clang -o test libsafedrive/src/test.c -rpath dist-x86_64-apple-darwin/lib -Ldist-x86_64-apple-darwin/lib -Idist-x86_64-apple-darwin/include -lz -lsodium -lsafedrive -framework Foundation -framework Security -O2

time RUST_BACKTRACE=1 ./test

