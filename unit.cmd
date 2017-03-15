IF [%ARCH%]==[] set ARCH=x86
IF [%TARGET%]==[] set TARGET=i686-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v140
IF [%LINKTYPE%]==[] set LINKTYPE=dll
IF [%CHANNEL%]==[] set CHANNEL=nightly
IF [%RUST_PINNED%]==[] set RUST_PINNED=beta-2017-03-03

ECHO testing SafeDrive for Windows-%ARCH%

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib
set SODIUM_STATIC=""
set RUST_BACKTRACE="1"

IF "%LINKTYPE%"=="mt" (
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
)

rustup.exe override set %RUST_PINNED%-%TARGET%

cargo.exe test --verbose --release -p libsafedrive --target %TARGET%