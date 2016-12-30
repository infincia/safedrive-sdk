ECHO testing SafeDrive for Windows-%ARCH%

set SODIUM_LIB_DIR=%CD%\dep-%TARGET%-%TOOLSET%\lib
set SODIUM_STATIC=""

set SQLITE3_LIB_DIR=%CD%\dep-%TARGET%-%TOOLSET%\lib

set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static

pushd libsafedrive
cargo.exe test --release --verbose
popd