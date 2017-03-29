IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141
IF [%LINKTYPE%]==[] set LINKTYPE=dll

ECHO building SafeDrive for Windows-%ARCH%

del /q dist-%TARGET%-%TOOLSET%-%LINKTYPE%

mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib
set RUST_BACKTRACE="1"

IF "%LINKTYPE%"=="mt" (
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
    set SODIUM_STATIC=""
)

call rustver.bat

rustup override set %RUST_VER%

cargo.exe build --release -p libsafedrive --target %TARGET%
cheddar -f libsafedrive\src\c_api.rs dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include\sddk.h

cargo.exe build --release -p safedrive --target %TARGET%

copy /y dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib\libsodium.dll %CD%\dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\libsodium.dll
copy /y dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib\libsodium.lib %CD%\dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\libsodium.lib

copy /y target\%TARGET%\release\safedrive.lib dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.lib
copy /y target\%TARGET%\release\safedrive.dll dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll

copy /y target\%TARGET%\release\safedrive.exe dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\

copy /y dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\libsodium.dll dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\libsodium.dll