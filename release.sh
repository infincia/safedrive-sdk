#!/usr/bin/env bash

set -e

export PATH=${HOME}/.cargo/bin:${PATH}

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Building release for ${TARGET}"


export DIST_PREFIX=${PWD}/target/${TARGET}/release
export BUILD_PREFIX=${DIST_PREFIX}/deps


export RUSTFLAGS=""
export CARGO_INCREMENTAL=1
export CFLAGS=""
export CPPFLAGS=""
export LDFLAGS=""


export OSX_VERSION_MIN="10.9"
export OSX_CPU_ARCH="core2"
export MAC_ARGS="-arch x86_64 -mmacosx-version-min=${OSX_VERSION_MIN} -march=${OSX_CPU_ARCH} -fapplication-extension"

case ${TARGET} in
    x86_64-apple-darwin)
        export CFLAGS="${CFLAGS} ${MAC_ARGS}"
        export CPPFLAGS="${CPPFLAGS} ${MAC_ARGS}"
        export LDFLAGS="${LDFLAGS} ${MAC_ARGS}"
        export RUSTFLAGS="${RUSTFLAGS} -C link-arg=-fapplication-extension -C link-arg=-mmacosx-version-min=${OSX_VERSION_MIN}"
        export OPENSSL_DIR=${BUILD_PREFIX}
        export DEP_OPENSSL_LIBRESSL=1
        ;;
    x86_64-unknown-linux-gnu)
        export CFLAGS="${CFLAGS}"
        export CPPFLAGS="${CPPFLAGS}"
        export LDFLAGS="${LDFLAGS}"
        ;;
    x86_64-unknown-linux-musl)
        export CFLAGS="${CFLAGS}"
        export CPPFLAGS="${CPPFLAGS}"
        export LDFLAGS="${LDFLAGS}"
        export CC=musl-gcc
        ;;
    wasm32-unknown-emscripten)
        export CPPFLAGS="${CPPFLAGS}"
        export CFLAGS="-Os"
        export CC=emcc
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    *)
        ;;
esac

mkdir -p ${DIST_PREFIX}/lib
mkdir -p ${DIST_PREFIX}/include
mkdir -p ${DIST_PREFIX}/bin

echo "Building dependencies for ${TARGET}"

bash dep.sh

RUST_VER_FILE=$(dirname $0)/rust-toolchain

RUST_VER=$(<${RUST_VER_FILE})

rustup override set ${RUST_VER}

case ${TARGET} in
    wasm32-unknown-emscripten)
        echo "Building sddk for ${TARGET}"

        RUST_BACKTRACE=1 cargo build --release -p sddk --target ${TARGET} > /dev/null

        ;;
    x86_64-unknown-linux-musl)

        ;;
    x86_64-unknown-linux-gnu)

        ;;
    *)
        echo "Building safedrive CLI for ${TARGET}"

        RUST_BACKTRACE=1 cargo build --release -p safedrive --target ${TARGET} > /dev/null

        echo "Building safedrive askpass for ${TARGET}"

        RUST_BACKTRACE=1 cargo build --release -p askpass --target ${TARGET} > /dev/null
        ;;
esac

echo "Copying build artifacts for ${TARGET}"

case ${TARGET} in
    x86_64-apple-darwin)
        install_name_tool -id "@rpath/libsddk.dylib" ${DIST_PREFIX}/libsddk.dylib
        cp -a target/${TARGET}/release/safedrive ${DIST_PREFIX}/io.safedrive.SafeDrive.cli
        cp -a target/${TARGET}/release/askpass ${DIST_PREFIX}/io.safedrive.SafeDrive.askpass
        ;;
    x86_64-unknown-linux-musl)
        cp -a target/${TARGET}/release/rsync ${DIST_PREFIX}/
        #cp -a target/${TARGET}/release/safedrive ${DIST_PREFIX}/
        ;;
    x86_64-unknown-linux-gnu)
        cp -a target/${TARGET}/release/rsync ${DIST_PREFIX}/
        #cp -a target/${TARGET}/release/safedrive ${DIST_PREFIX}/
        ;;
    wasm32-unknown-emscripten)
        ;;
    *)
        ;;
esac
