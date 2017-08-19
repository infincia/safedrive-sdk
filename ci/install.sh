#!/usr/bin/env bash

# `install` phase: install stuff needed for the `script` phase

set -ex

export PATH=${HOME}/.cargo/bin:${PATH}

. $(dirname $0)/utils.sh

RUST_VER_FILE=$(dirname $0)/../rustver.conf

install_fuse() {
    case "${TRAVIS_OS_NAME}" in
        linux)
            ;;
        osx)
            brew cask install osxfuse || true
            ;;
    esac
}

install_cmake() {
    case "${TRAVIS_OS_NAME}" in
        linux)
            ;;
        osx)
            brew install cmake || true
            brew upgrade cmake || true
            ;;
    esac
}

install_rustup() {
    RUST_VER=$(<${RUST_VER_FILE})

    echo "Using Rust ${RUST_VER}"
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=${RUST_VER}
    if [ -z "${TARGET}" ]; then
        export TARGET=`rustup show | awk 'match($0, /Default host: ([0-9a-zA-Z\_]).+/) { ver = substr($3, RSTART, RLENGTH); print ver;}'`
    fi
    rustup target add ${TARGET} || true
    rustc -V
    cargo -V
    if [ ! -f ${HOME}/.cargo/bin/cheddar ]; then
        cargo install moz-cheddar
    else
        echo "cheddar already installed, skipping"
    fi
}

main() {
    install_cmake
    install_fuse
    install_rustup
}

main
