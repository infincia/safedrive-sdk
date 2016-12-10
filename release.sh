#!/bin/sh

rm -rf dist-osx

export SODIUM_LIB_DIR=dep-osx/lib
export SODIUM_STATIC

export SQLITE3_LIB_DIR=dep-osx/lib


export OPENSSL_DIR=$PWD/dep-osx
export OPENSSL_STATIC

cargo build --release --verbose

mkdir -p dist-osx/lib
mkdir -p dist-osx/include

cp -a target/release/libsdsync.a dist-osx/lib/

cp -a dep-osx/lib/* dist-osx/lib/
cp -a dep-osx/include/* dist-osx/include/
