#!/usr/bin/env bash

case $TARGET in
    # configure emulation for transparent execution of foreign binaries
    aarch64-unknown-linux-gnu)
        export QEMU_LD_PREFIX=/usr/aarch64-linux-gnu
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CONFIGURE_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="shared"
        ;;
    arm*-unknown-linux-gnueabihf)
        export QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CONFIGURE_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="shared"
        ;;
    x86_64-apple-darwin)
        export OSX_VERSION_MIN=${OSX_VERSION_MIN-"10.9"}
        export OSX_CPU_ARCH=${OSX_CPU_ARCH-"core2"}
        export CFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -O2 -g -flto"
        export LDFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -flto"
        export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"
        export CONFIGURE_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="shared"
        ;;
    x86_64-unknown-linux-gnu|i686-unknown-linux-gnu)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CONFIGURE_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="shared"
        ;;
    x86_64-unknown-linux-musl|i686-unknown-linux-musl)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CC=musl-gcc
        export CONFIGURE_ARGS="--enable-shared=no"
        export OPENSSL_ARGS="no-shared"
        ;;
    *)
        ;;
esac

if [ ! -z "$QEMU_LD_PREFIX" ]; then
    # Run tests on a single thread when using QEMU user emulation
    export RUST_TEST_THREADS=1
fi

if [ ! -f dep/$TARGET/lib/libssl.a ]; then
    export OPENSSL_VER=1.1.0d

    wget https://www.openssl.org/source/openssl-$OPENSSL_VER.tar.gz
    tar xvzf openssl-$OPENSSL_VER.tar.gz
    OPENSSL_PREFIX=$PWD/dep/$TARGET
    pushd openssl-$OPENSSL_VER
    ./config --prefix=$OPENSSL_PREFIX $OPENSSL_ARGS
    env C_INCLUDE_PATH=$OPENSSL_PREFIX/include/ make depend
    make
    make install
    popd
    rm -rf openssl*
fi

if [ ! -f dep/$TARGET/lib/libsodium.a ]; then
    export SODIUM_VER=1.0.11

    wget https://github.com/jedisct1/libsodium/releases/download/$SODIUM_VER/libsodium-$SODIUM_VER.tar.gz
    tar xvfz libsodium-$SODIUM_VER.tar.gz
    SODIUM_PREFIX=$PWD/dep/$TARGET
    pushd libsodium-$SODIUM_VER
    ./configure --prefix=$SODIUM_PREFIX $CONFIGURE_ARGS
    make
    make install
    popd
    rm -rf libsodium*
fi