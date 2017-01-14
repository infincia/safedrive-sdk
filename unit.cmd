IF [%ARCH%]==[] set ARCH=x86
IF [%TARGET%]==[] set TARGET=i686-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v140
IF [%LINKTYPE%]==[] set LINKTYPE=dll
IF [%CHANNEL%]==[] set CHANNEL=nightly

ECHO testing SafeDrive for Windows-%ARCH%

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib

IF "%LINKTYPE%"=="mt" (
    set SODIUM_STATIC=""
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
)

pushd libsafedrive
cargo.exe test --release --verbose
popd