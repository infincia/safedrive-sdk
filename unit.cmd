ECHO testing SafeDrive for Windows-%ARCH%

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib

IF "%LINKTYPE%"=="mt" (
    set SODIUM_STATIC=""
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
)

pushd libsafedrive
cargo.exe test --release --verbose
popd