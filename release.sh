#!/bin/sh

set -ex

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
    *)
        ;;
esac

if [ ! -z "$QEMU_LD_PREFIX" ]; then
    # Run tests on a single thread when using QEMU user emulation
    export RUST_TEST_THREADS=1
fi

rm -rf dist-$TARGET
mkdir -p dist-$TARGET/lib
mkdir -p dist-$TARGET/include
mkdir -p dist-$TARGET/bin
mkdir -p dist-$TARGET/dep

export SODIUM_LIB_DIR=$PWD/dep-$TARGET/lib
export SODIUM_STATIC

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