#!/usr/bin/env bash

case $TARGET in
    x86_64-apple-darwin)
        export OSX_VERSION_MIN=${OSX_VERSION_MIN-"10.9"}
        export OSX_CPU_ARCH=${OSX_CPU_ARCH-"core2"}
        export CFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -O2 -g -flto"
        export LDFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -flto"
        export RUSTFLAGS="-C link-args=-mmacosx-version-min=10.9"
        export SODIUM_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="no-deprecated shared no-ssl3 no-weak-ssl-ciphers no-engine no-afalgeng no-async"
        ;;
    x86_64-unknown-linux-gnu|i686-unknown-linux-gnu)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export SODIUM_ARGS="--enable-shared=yes"
        export OPENSSL_ARGS="no-deprecated shared no-ssl3 no-weak-ssl-ciphers no-engine no-afalgeng no-async"
        ;;
    x86_64-unknown-linux-musl|i686-unknown-linux-musl)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CC=musl-gcc
        export SODIUM_ARGS="--enable-shared=no"
        export OPENSSL_ARGS="no-deprecated no-shared no-ssl3 no-weak-ssl-ciphers no-engine no-afalgeng no-async"
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

    wget https://www.openssl.org/source/openssl-$OPENSSL_VER.tar.gz > /dev/null
    tar xvzf openssl-$OPENSSL_VER.tar.gz > /dev/null
    OPENSSL_PREFIX=$PWD/dep/$TARGET
    pushd openssl-$OPENSSL_VER
    ./config --prefix=$OPENSSL_PREFIX --openssldir=/etc/ssl $OPENSSL_ARGS > /dev/null
    make clean > /dev/null
    make > /dev/null
    make install > /dev/null
    popd
    rm -rf openssl*
fi

if [ ! -f dep/$TARGET/lib/libsodium.a ]; then
    export SODIUM_VER=1.0.11

    wget https://github.com/jedisct1/libsodium/releases/download/$SODIUM_VER/libsodium-$SODIUM_VER.tar.gz > /dev/null
    tar xvfz libsodium-$SODIUM_VER.tar.gz > /dev/null
    SODIUM_PREFIX=$PWD/dep/$TARGET
    pushd libsodium-$SODIUM_VER
    ./configure --prefix=$SODIUM_PREFIX $SODIUM_ARGS > /dev/null
    make clean > /dev/null
    make > /dev/null
    make install > /dev/null
    popd
    rm -rf libsodium*
fi