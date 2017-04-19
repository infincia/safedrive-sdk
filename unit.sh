#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Testing for $TARGET"

export RUSTFLAGS="-Z print-link-args"

case ${TARGET} in
    x86_64-apple-darwin)
        export OSX_VERSION_MIN=${OSX_VERSION_MIN-"10.9"}
        export OSX_CPU_ARCH=${OSX_CPU_ARCH-"core2"}
        export CFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -O2 -g -flto"
        export LDFLAGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -flto"
        export RUSTFLAGS="${RUSTFLAGS} -C link-args=-mmacosx-version-min=10.9 "
        ;;
    x86_64-unknown-linux-gnu)
        export CFLAGS="-O2 -g -flto -I${BUILD_PREFIX}/include"
        export LDFLAGS="-flto -L${BUILD_PREFIX}/lib"
        ;;
    i686-unknown-linux-gnu)
        export CFLAGS="-O2 -g -flto -m32"
        export LDFLAGS="-flto -L${BUILD_PREFIX}/lib"
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    x86_64-unknown-linux-musl)
        export CFLAGS="-O2 -g -flto"
        export LDFLAGS="-flto"
        export CC=musl-gcc
        ;;
    i686-unknown-linux-musl)
        export CFLAGS="-O2 -g -flto -m32"
        export LDFLAGS="-flto"
        export CC=musl-gcc
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    *)
        ;;
esac


bash dep.sh

source ./rustver.sh

rustup override set $RUST_VER

RUST_BACKTRACE=1 cargo test --release -p libsafedrive --target $TARGET