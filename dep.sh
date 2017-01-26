#!/usr/bin/env bash

case $TARGET in
    # configure emulation for transparent execution of foreign binaries
    aarch64-unknown-linux-gnu)
        export QEMU_LD_PREFIX=/usr/aarch64-linux-gnu
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CONFIGURE_ARGS="--enable-shared=yes"
        ;;
    arm*-unknown-linux-gnueabihf)
        export QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CONFIGURE_ARGS="--enable-shared=yes"
        ;;
    x86_64-apple-darwin)
        export OSX_VERSION_MIN=${OSX_VERSION_MIN-"10.9"}
        export OSX_CPU_ARCH=${OSX_CPU_ARCH-"core2"}
        export CFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -O2 -g -flto"
        export LDFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -flto"
        export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"
        export CONFIGURE_ARGS="--enable-shared=yes"
        ;;
    x86_64-unknown-linux-gnu|i686-unknown-linux-gnu)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CONFIGURE_ARGS="--enable-shared=yes"
        ;;
    x86_64-unknown-linux-musl|i686-unknown-linux-musl)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CC=musl-gcc
        export CONFIGURE_ARGS="--enable-shared=no"
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
    SODIUM_PREFIX=$PWD/dep/$TARGET
    pushd libsodium-1.0.11
    ./configure --prefix=$SODIUM_PREFIX $CONFIGURE_ARGS
    make
    make install
    popd
    rm -rf libsodium*
fi