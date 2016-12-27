#!/bin/sh

export SODIUM_LIB_DIR=dep-osx/lib
export SODIUM_STATIC

export SQLITE3_LIB_DIR=dep-osx/lib


export OPENSSL_DIR=$PWD/dep-osx
export OPENSSL_STATIC

export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"

cargo test --verbose