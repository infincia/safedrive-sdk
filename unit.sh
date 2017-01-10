#!/bin/sh

set -ex

rustup default nightly

TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`

case $TARGET in
    # configure emulation for transparent execution of foreign binaries
    aarch64-unknown-linux-gnu)
        export QEMU_LD_PREFIX=/usr/aarch64-linux-gnu
        ;;
    arm*-unknown-linux-gnueabihf)
        export QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf
        ;;
    x86_64-apple-darwin)
        export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"
        ;;
    *-linux-unknown)
        wget https://github.com/jedisct1/libsodium/releases/download/1.0.11/libsodium-1.0.11.tar.gz
        tar xvfz libsodium-1.0.11.tar.gz
        SODIUM_PREFIX=$PWD/dep-$TARGET
        pushd libsodium-1.0.11
        ./configure --prefix=$SODIUM_PREFIX
        make
        make install
        popd
        ;;
    *)
        ;;
esac

if [ ! -z "$QEMU_LD_PREFIX" ]; then
    # Run tests on a single thread when using QEMU user emulation
    export RUST_TEST_THREADS=1
fi

export SODIUM_LIB_DIR=$PWD/dep-$TARGET/lib
export SODIUM_STATIC

pushd libsafedrive
cargo test --release --verbose
popd