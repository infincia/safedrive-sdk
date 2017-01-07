#!/bin/sh

set -e

TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`


rm -rf dist-$TARGET
mkdir -p dist-$TARGET/lib
mkdir -p dist-$TARGET/include
mkdir -p dist-$TARGET/bin
mkdir -p dist-$TARGET/dep

export SODIUM_LIB_DIR=$PWD/dep-$TARGET/lib
export SODIUM_STATIC

export OPENSSL_DIR=$PWD/dep-$TARGET
export OPENSSL_STATIC

export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"

pushd libsafedrive
cargo build --release --verbose
popd
pushd safedrive
cargo build --release --verbose
popd



cp -a dep-$TARGET/* dist-$TARGET/dep/

cp -a target/release/deps/libsafedrive.a dist-$TARGET/lib/
cp -a target/release/deps/libsafedrive.dylib dist-$TARGET/lib/
cp -a target/release/safedrive dist-$TARGET/bin/