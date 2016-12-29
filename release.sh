#!/bin/sh

set -e

TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`


rm -rf dist-$TARGET

export SODIUM_LIB_DIR=dep-$TARGET/lib
export SODIUM_STATIC

export SQLITE3_LIB_DIR=dep-$TARGET/lib


export OPENSSL_DIR=$PWD/dep-$TARGET
export OPENSSL_STATIC

export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"

pushd libsafedrive
cargo build --release --verbose
popd
pushd safedrive
cargo build --release --verbose
popd

mkdir -p dist-$TARGET/lib
mkdir -p dist-$TARGET/bin
mkdir -p dist-$TARGET/dep

cp -a dep-$TARGET/* dist-$TARGET/dep/

cp -a target/release/libsafedrive.a dist-$TARGET/lib/
cp -a target/release/safedrive dist-$TARGET/bin/