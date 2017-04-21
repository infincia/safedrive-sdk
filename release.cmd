IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%LINKTYPE%]==[] set LINKTYPE=static

ECHO building safedrive for %TARGET% (%TOOLSET%-%LINKTYPE%)

del /q dist-%TARGET%-%TOOLSET%-%LINKTYPE%

mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin

set NATIVE_BUILD_PREFIX=dep\%TARGET%\%TOOLSET%\%LINKTYPE%

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib
set SODIUM_STATIC=""
set RUST_BACKTRACE="1"
set RUST_FLAGS=""

call dep.cmd

call rustver.bat

rustup override set %RUST_VER%

cargo.exe build --release -p libsafedrive --target %TARGET%
cheddar -f "libsafedrive\src\c_api.rs" "dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include\sddk.h"

cargo.exe build --release -p safedrive --target %TARGET%
cargo.exe build --release -p safedrived --target %TARGET%

ECHO copying "target\%TARGET%\release\safedrive.dll" "dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll"
copy /y "target\%TARGET%\release\safedrive.dll" "dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll"

ECHO copying "target\%TARGET%\release\safedrive.exe" "dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\"
copy /y "target\%TARGET%\release\safedrive.exe" "dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\"

ECHO copying "target\%TARGET%\release\safedrived.exe" "dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\"
copy /y "target\%TARGET%\release\safedrived.exe" "dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\"

