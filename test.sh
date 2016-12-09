#!/bin/sh

rm test -f &> /dev/null

clang -o test src/test.c -Ldist-osx/lib -Idist-osx/include -lsqlite3 -lz -lsodium  -lsdsync -framework Foundation -framework Security -O2

time RUST_BACKTRACE=1 ./test

