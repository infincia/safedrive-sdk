#!/bin/sh

set -ex

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Building for $TARGET"

rm -rf dist-$TARGET
mkdir -p dist-$TARGET/lib
mkdir -p dist-$TARGET/include
mkdir -p dist-$TARGET/bin

bash dep.sh


cargo build --release --verbose -p safedrive

# build safedrived on linux only
case $TARGET in
    x86_64-apple-darwin)
        ;;
    *)
        cargo build --release --verbose -p safedrived
        ;;
esac

cheddar -f libsafedrive/src/c_api.rs dist-$TARGET/include/sddk.h


cp -a dep/$TARGET/lib/* dist-$TARGET/lib/

cp -a target/release/libsafedrive.a dist-$TARGET/lib/libsafedrive.a
cp -a target/release/libsafedrive.dylib dist-$TARGET/lib/libsafedrive.dylib || true
install_name_tool -id "@rpath/libsafedrive.dylib" dist-$TARGET/lib/libsafedrive.dylib || true
install_name_tool -id "@rpath/libsodium.18.dylib" dist-$TARGET/lib/libsodium.18.dylib || true
cp -a target/release/libsafedrive.so dist-$TARGET/lib/libsafedrive.so || true
cp -a target/release/safedrive dist-$TARGET/bin/
cp -a target/release/safedrived dist-$TARGET/bin/ || true