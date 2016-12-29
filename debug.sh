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
cargo build --verbose
popd
pushd safedrive
cargo build --verbose
popd

mkdir -p dist/lib
mkdir -p dist/bin
mkdir -p dist/dep

cp -a dep-osx/* dist/dep/

cp -a target/debug/libsafedrive.a dist/lib/libsafedrive.a
cp -a target/debug/safedrive dist/bin/