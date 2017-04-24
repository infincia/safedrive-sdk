#!/usr/bin/env bash

# `install` phase: install stuff needed for the `script` phase

set -ex

. $(dirname $0)/utils.sh
. $(dirname $0)/../rustver.sh

install_fuse() {
    case "$TRAVIS_OS_NAME" in
        linux)
            ;;
        osx)
            brew cask install osxfuse
            ;;
    esac
}

install_rustup() {
    echo "Using Rust $RUST_VER"
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$RUST_VER
    rustup target add $TARGET || true
    rustc -V
    cargo -V
    cargo install rusty-cheddar
}

main() {
    install_fuse
    install_rustup
}

main
