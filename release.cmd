IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141
IF [%LINKTYPE%]==[] set LINKTYPE=dll

set LIBPREFIX=
IF [%TOOLSET%]==[v141] set LIBPREFIX=lib



ECHO building SafeDrive for Windows-%ARCH%

del /q dist-%TARGET%-%TOOLSET%-%LINKTYPE%

mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib
set RUST_BACKTRACE="1"

IF "%LINKTYPE%"=="static" (
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
    set SODIUM_STATIC=""
)

call rustver.bat

rustup override set %RUST_VER%

cargo.exe build --release -p libsafedrive --target %TARGET%
cheddar -f libsafedrive\src\c_api.rs dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include\sddk.h

cargo.exe build --release -p safedrive --target %TARGET%

copy /y dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib\%LIBPREFIX%sodium.dll dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\%LIBPREFIX%sodium.dll
copy /y dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib\%LIBPREFIX%sodium.lib dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\%LIBPREFIX%sodium.lib

copy /y target\%TARGET%\release\safedrive.dll dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\%LIBPREFIX%safedrive.dll

copy /y target\%TARGET%\release\safedrive.exe dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\

copy /y dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\%LIBPREFIX%sodium.dll dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\%LIBPREFIX%sodium.dll
