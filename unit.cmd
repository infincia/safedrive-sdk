setlocal enabledelayedexpansion

IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%LINKTYPE%]==[] set LINKTYPE=static

ECHO Testing release for %TARGET% (%TOOLSET%-%LINKTYPE%)

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib
set SODIUM_STATIC=""
set CARGO_INCREMENTAL="1"
set RUST_BACKTRACE="1"
set RUST_FLAGS=""

ECHO Building test dependencies for %TARGET% (%TOOLSET%-%LINKTYPE%)

call dep.cmd || goto :error

call rustver.bat

rustup override set %RUST_VER%

ECHO Testing libsafedrive for %TARGET% (%TOOLSET%-%LINKTYPE%)

cargo.exe test --verbose --release -p libsafedrive --target %TARGET% || goto :error
goto :EOF

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!
