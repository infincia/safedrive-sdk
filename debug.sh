#!/bin/sh

set -ex

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Building for $TARGET"

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

rm -rf dist-$TARGET
mkdir -p dist-$TARGET/lib
mkdir -p dist-$TARGET/include
mkdir -p dist-$TARGET/bin

if [ ! -f dep/$TARGET/lib/libsodium.a ]; then
    wget https://github.com/jedisct1/libsodium/releases/download/1.0.11/libsodium-1.0.11.tar.gz
    tar xvfz libsodium-1.0.11.tar.gz
    SODIUM_PREFIX=$PWD/dep/$TARGET
    pushd libsodium-1.0.11
    ./configure --prefix=$SODIUM_PREFIX --enable-shared=yes
    make
    make install
    popd
    rm -rf libsodium*
fi

export SODIUM_LIB_DIR=$PWD/dep/$TARGET/lib
export SODIUM_STATIC

pushd libsafedrive
cargo build --verbose
popd
pushd safedrive
cargo build --verbose
popd



cp -a dep/$TARGET/* dist-$TARGET/

cp -a target/debug/deps/libsafedrive*.a dist-$TARGET/lib/libsafedrive.a
cp -a target/debug/deps/libsafedrive*.dylib dist-$TARGET/lib/libsafedrive.dylib || true
cp -a target/debug/deps/libsafedrive*.so dist-$TARGET/lib/libsafedrive.so || true
cp -a target/debug/safedrive dist-$TARGET/bin/