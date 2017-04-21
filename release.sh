#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Building for $TARGET"

export BUILD_PREFIX=$PWD/dep/${TARGET}

export RUSTFLAGS=""
export CFLAGS="-O2 -g -I${BUILD_PREFIX}/include"
export CPPFLAGS="-O2 -g -I${BUILD_PREFIX}/include"
export LDFLAGS="-flto -L${BUILD_PREFIX}/lib"

export OSX_VERSION_MIN="10.9"
export OSX_CPU_ARCH="core2"
export MAC_ARGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH}"

case ${TARGET} in
    x86_64-apple-darwin)
        export CFLAGS="${CFLAGS} ${MAC_ARGS}"
        export CPPFLAGS="${CPPFLAGS} ${MAC_ARGS}"
        export LDFLAGS="${LDFLAGS} ${MAC_ARGS}"
        export RUSTFLAGS="${RUSTFLAGS} -C link-args=-mmacosx-version-min=${OSX_VERSION_MIN}"
        ;;
    x86_64-unknown-linux-gnu)
        export CFLAGS="${CFLAGS}"
        export CPPFLAGS="${CPPFLAGS}"
        export LDFLAGS="${LDFLAGS}"
        ;;
    i686-unknown-linux-gnu)
        export CFLAGS="${CFLAGS} -m32"
        export CPPFLAGS="${CPPFLAGS} -m32"
        export LDFLAGS="${LDFLAGS}"
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    x86_64-unknown-linux-musl)
        export CFLAGS="${CFLAGS}"
        export CPPFLAGS="${CPPFLAGS}"
        export LDFLAGS="${LDFLAGS}"
        export CC=musl-gcc
        ;;
    i686-unknown-linux-musl)
        export CFLAGS="${CFLAGS} -m32"
        export CPPFLAGS="${CPPFLAGS} -m32"
        export LDFLAGS="${LDFLAGS}"
        export CC=musl-gcc
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    *)
        ;;
esac

rm -rf dist-$TARGET
mkdir -p dist-$TARGET/lib
mkdir -p dist-$TARGET/include
mkdir -p dist-$TARGET/bin

bash dep.sh

source ./rustver.sh

rustup override set $RUST_VER

RUST_BACKTRACE=1 cargo build --release -p safedrive --target $TARGET > /dev/null
RUST_BACKTRACE=1 cargo build --release -p safedrived --target $TARGET > /dev/null

cheddar -f libsafedrive/src/c_api.rs dist-$TARGET/include/sddk.h


case $TARGET in
    x86_64-apple-darwin)
        cp -a target/$TARGET/release/libsafedrive.dylib dist-$TARGET/lib/libsafedrive.dylib
        install_name_tool -id "@rpath/libsafedrive.dylib" dist-$TARGET/lib/libsafedrive.dylib
        cp -a target/$TARGET/release/safedrived dist-$TARGET/bin/io.safedrive.SafeDrive.daemon
        cp -a target/$TARGET/release/safedrive dist-$TARGET/bin/io.safedrive.SafeDrive.cli
        ;;
    i686-unknown-linux-musl|x86_64-unknown-linux-musl)
        cp -a target/$TARGET/release/libsafedrive.so dist-$TARGET/lib/libsafedrive.so
        cp -a target/$TARGET/release/safedrived dist-$TARGET/bin/
        cp -a target/$TARGET/release/safedrive dist-$TARGET/bin/
        ;;
    i686-unknown-linux-gnu|x86_64-unknown-linux-gnu)
        cp -a dep/$TARGET/lib/pkgconfig dist-$TARGET/lib/
        cp -a dep/$TARGET/lib/*.so dist-$TARGET/lib/
        cp -a dep/$TARGET/lib/.*_ver dist-$TARGET/lib/
        cp -a target/$TARGET/release/libsafedrive.so dist-$TARGET/lib/libsafedrive.so
        cp -a target/$TARGET/release/safedrived dist-$TARGET/bin/
        cp -a target/$TARGET/release/safedrive dist-$TARGET/bin/
        ;;
    *)
        ;;
esac
