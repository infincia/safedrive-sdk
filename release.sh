#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Building release for ${TARGET}"

export BUILD_PREFIX=$PWD/dep/${TARGET}
export DIST_PREFIX=$PWD/dist/${TARGET}
export INTR_PREFIX=$PWD/build/${TARGET}
export CMAKE_PREFIX=$PWD/Windows

export RUSTFLAGS=""
export CARGO_INCREMENTAL=1
export CFLAGS=""
export CPPFLAGS=""
export LDFLAGS=""
export OPENSSL_DIR=${BUILD_PREFIX}

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
    wasm32-unknown-emscripten)
        export CPPFLAGS="${CPPFLAGS}"
        export CFLAGS="-Os"
        export CC=emcc
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    *)
        ;;
esac

rm -rf ${DIST_PREFIX}
mkdir -p ${DIST_PREFIX}/lib
mkdir -p ${DIST_PREFIX}/include
mkdir -p ${DIST_PREFIX}/bin
mkdir -p ${INTR_PREFIX}
mkdir -p ${INTR_PREFIX}/Release
mkdir -p ${INTR_PREFIX}/Debug


echo "Building dependencies for ${TARGET}"

bash dep.sh

source ./rustver.sh

rustup override set ${RUST_VER}

case ${TARGET} in
    wasm32-unknown-emscripten)
            echo "Building sddk for ${TARGET}"

        RUST_BACKTRACE=1 cargo build --release -p sddk --target ${TARGET} > /dev/null

        ;;
    *)
        echo "Building safedrive CLI for ${TARGET}"

        RUST_BACKTRACE=1 cargo build --release -p safedrive --target ${TARGET} > /dev/null

        echo "Building safedrive daemon for ${TARGET}"

        RUST_BACKTRACE=1 cargo build --release -p safedrived --target ${TARGET} > /dev/null
        ;;
esac

echo "Building SDDK headers for ${TARGET}"

cheddar -f sddk/src/c_api.rs ${DIST_PREFIX}/include/sddk.h

echo "Copying build artifacts for ${TARGET}"

case ${TARGET} in
    x86_64-apple-darwin)
        #cp -a target/${TARGET}/release/libsddk.dylib ${DIST_PREFIX}/lib/libsddk.dylib
        #install_name_tool -id "@rpath/libsddk.dylib" ${DIST_PREFIX}/lib/libsddk.dylib
        cp -a target/${TARGET}/release/libsddk.a ${DIST_PREFIX}/lib/libsddk.a
        cp -a target/${TARGET}/release/safedrived ${DIST_PREFIX}/bin/io.safedrive.SafeDrive.daemon
        cp -a target/${TARGET}/release/safedrive ${DIST_PREFIX}/bin/io.safedrive.SafeDrive.cli
        ;;
    i686-unknown-linux-musl|x86_64-unknown-linux-musl)
        cp -a target/${TARGET}/release/libsddk.a ${DIST_PREFIX}/lib/libsddk.a
        cp -a target/${TARGET}/release/safedrived ${DIST_PREFIX}/bin/
        cp -a target/${TARGET}/release/safedrive ${DIST_PREFIX}/bin/
        ;;
    i686-unknown-linux-gnu|x86_64-unknown-linux-gnu)
        cp -a target/${TARGET}/release/libsddk.a ${DIST_PREFIX}/lib/libsddk.a
        #cp -a target/${TARGET}/release/libsddk.so ${DIST_PREFIX}/lib/libsddk.so
        cp -a target/${TARGET}/release/safedrived ${DIST_PREFIX}/bin/
        cp -a target/${TARGET}/release/safedrive ${DIST_PREFIX}/bin/
        ;;
    wasm32-unknown-emscripten)
        cp -a target/${TARGET}/release/libsddk.wasm ${DIST_PREFIX}/lib/libsddk.wasm
        cp -a target/${TARGET}/release/libsddk.js ${DIST_PREFIX}/lib/libsddk.js
        cp -a target/${TARGET}/release/libsddk.html ${DIST_PREFIX}/lib/libsddk.html
        ;;
    *)
        ;;
esac

echo "Building C++ SDK for ${TARGET}"

pushd "${INTR_PREFIX}"

case ${TARGET} in
    x86_64-apple-darwin)
        cmake "${CMAKE_PREFIX}" -G Xcode -D"TARGET=${TARGET}" -D"CONFIGURATION=${CONFIGURATION}"
        xcodebuild

        echo copying ${INTR_PREFIX}/${CONFIGURATION}/libSafeDriveSDK.a to ${DIST_PREFIX}/lib/
        cp -a ${INTR_PREFIX}/${CONFIGURATION}/libSafeDriveSDK.a ${DIST_PREFIX}/lib/
        ;;
    *)
        #cmake "${CMAKE_PREFIX}" -G"Unix Makefiles" -D"TARGET=${TARGET}" -D"CONFIGURATION=${CONFIGURATION}"
        #make
        ;;
esac

popd
