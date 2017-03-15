#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Building for $TARGET"

case $TARGET in
    i686-unknown-linux-gnu|i686-unknown-linux-musl)
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
export DEP_OPENSSL_VERSION="110"

rustup override set beta-2017-03-03-$TARGET

RUST_BACKTRACE=1 cargo build --release -p safedrive --target $TARGET > /dev/null

# build safedrived on linux only
case $TARGET in
    x86_64-apple-darwin)
        ;;
    *)
        RUST_BACKTRACE=1 cargo build --release -p safedrived --target $TARGET > /dev/null
        ;;
esac

cheddar -f libsafedrive/src/c_api.rs dist-$TARGET/include/sddk.h


cp -a dep/$TARGET/lib/* dist-$TARGET/lib/

cp -a target/$TARGET/release/libsafedrive.a dist-$TARGET/lib/libsafedrive.a
cp -a target/$TARGET/release/libsafedrive.dylib dist-$TARGET/lib/libsafedrive.dylib || true
install_name_tool -id "@rpath/libsafedrive.dylib" dist-$TARGET/lib/libsafedrive.dylib || true
install_name_tool -id "@rpath/libsodium.18.dylib" dist-$TARGET/lib/libsodium.18.dylib || true
cp -a target/$TARGET/release/libsafedrive.so dist-$TARGET/lib/libsafedrive.so || true
cp -a target/$TARGET/release/safedrive dist-$TARGET/bin/
cp -a target/$TARGET/release/safedrived dist-$TARGET/bin/ || true