#!/bin/sh

set -ex

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

echo "Testing for $TARGET"

case $TARGET in
    i686-unknown-linux-gnu|i686-unknown-linux-musl)
        export PKG_CONFIG_ALLOW_CROSS=1
        ;;
    *)
        ;;
esac

bash dep.sh
export DEP_OPENSSL_VERSION="110"

cargo test --release -p libsafedrive --target $TARGET