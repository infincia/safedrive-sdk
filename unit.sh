#!/usr/bin/env bash

set -e

if [ -z "${TARGET}" ]; then
    export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
fi

if [ -z "${RUST_PINNED}" ]; then
    export RUST_PINNED=beta-2017-03-03
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

rustup override set $RUST_PINNED-$TARGET

RUST_BACKTRACE=1 cargo test --release -p libsafedrive --target $TARGET