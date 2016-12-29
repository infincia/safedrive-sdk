#!/bin/sh

TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`

export SODIUM_LIB_DIR=dep-$TARGET/lib
export SODIUM_STATIC

export SQLITE3_LIB_DIR=dep-$TARGET/lib


export OPENSSL_DIR=$PWD/dep-$TARGET
export OPENSSL_STATIC

export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"

pushd libsafedrive
cargo test --release --verbose
popd