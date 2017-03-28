IF [%ARCH%]==[] set ARCH=x86
IF [%TARGET%]==[] set TARGET=i686-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v140
IF [%LINKTYPE%]==[] set LINKTYPE=dll

ECHO building SafeDrive for Windows-%ARCH%

mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib
set SODIUM_STATIC=""
set RUST_BACKTRACE="1"

IF "%LINKTYPE%"=="mt" (
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
)

call rustver.bat

rustup override set %RUST_VER%

cargo.exe build --release -p safedrive --target %TARGET%
cheddar -f libsafedrive\src\c_api.rs dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include\sddk.h

robocopy %CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib\ %CD%\dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\ /COPYALL /E

copy /y target\%TARGET%\release\safedrive.lib dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.lib
copy /y target\%TARGET%\release\safedrive.dll.lib dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll.lib
copy /y target\%TARGET%\release\safedrive.dll.exp dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll.exp
copy /y target\%TARGET%\release\safedrive.dll dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll
copy /y target\%TARGET%\release\safedrive.pdb dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.pdb

copy /y target\%TARGET%\release\safedrive.exe dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\

