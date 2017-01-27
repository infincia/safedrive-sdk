# `install` phase: install stuff needed for the `script` phase

set -ex

. $(dirname $0)/utils.sh

install_rustup() {
    # uninstall the rust toolchain installed by travis, we are going to use rustup
    sh ~/rust/lib/rustlib/uninstall.sh
    echo "Using Rust $TRAVIS_RUST_VERSION"
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain=$TRAVIS_RUST_VERSION
    rustup target add $TARGET || true
    rustc -V
    cargo -V
    cargo install rusty-cheddar
}

main() {
    install_rustup
}

main
