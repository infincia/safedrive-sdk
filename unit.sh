#!/bin/sh

set -ex

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Testing for $TARGET"

case $TARGET in
    # configure emulation for transparent execution of foreign binaries
    aarch64-unknown-linux-gnu)
        export QEMU_LD_PREFIX=/usr/aarch64-linux-gnu
        ;;
    arm*-unknown-linux-gnueabihf)
        export QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf
        ;;
    x86_64-apple-darwin)
        export OSX_VERSION_MIN=${OSX_VERSION_MIN-"10.9"}
        export OSX_CPU_ARCH=${OSX_CPU_ARCH-"core2"}
        export CFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -O2 -g -flto"
        export LDFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -flto"
        export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"
        ;;
    x86_64-unknown-linux-gnu|i686-unknown-linux-gnu)
        ;;
    *)
        ;;
esac

if [ ! -z "$QEMU_LD_PREFIX" ]; then
    # Run tests on a single thread when using QEMU user emulation
    export RUST_TEST_THREADS=1
fi

if [ ! -f dep/$TARGET/lib/libsodium.a ]; then
    wget https://github.com/jedisct1/libsodium/releases/download/1.0.11/libsodium-1.0.11.tar.gz
    tar xvfz libsodium-1.0.11.tar.gz
    SODIUM_PREFIX=$PWD/dep-$TARGET
    pushd libsodium-1.0.11
    ./configure --prefix=$SODIUM_PREFIX --enable-shared=yes
    make
    make install
    popd
    rm -rf libsodium*
fi

export SODIUM_LIB_DIR=$PWD/dist-$TARGET/lib
export SODIUM_STATIC

pushd libsafedrive
cargo test --release --verbose
popd