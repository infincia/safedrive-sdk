IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%LINKTYPE%]==[] set LINKTYPE=static

ECHO testing safedrive for %TARGET% (%TOOLSET%-%LINKTYPE%)

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib
set RUST_BACKTRACE="1"

IF "%LINKTYPE%"=="static" (
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
    set SODIUM_STATIC=""
)

call rustver.bat

rustup.exe override set %RUST_VER%

cargo.exe test --verbose --release -p libsafedrive --target %TARGET%