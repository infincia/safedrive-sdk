#!/bin/sh

set -e

rm -rf dist

export SODIUM_LIB_DIR=dep-osx/lib
export SODIUM_STATIC

export SQLITE3_LIB_DIR=dep-osx/lib


export OPENSSL_DIR=$PWD/dep-osx
export OPENSSL_STATIC

export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"

pushd libsafedrive
cargo build --release --verbose
popd
pushd safedrive
cargo build --release --verbose
popd

mkdir -p dist/lib
mkdir -p dist/bin
mkdir -p dist/dep

cp -a dep-osx/* dist/dep/

cp -a target/release/libsafedrive.a dist/lib/
cp -a target/release/safedrive dist/bin/