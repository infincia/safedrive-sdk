#!/bin/sh


clang -o test src/test.c -Ldist-osx/lib -Idist-osx/include -lsqlite3 -lz -lsodium  -lsdsync -O2

time RUST_BACKTRACE=1 ./test

