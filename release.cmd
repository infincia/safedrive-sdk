setlocal enabledelayedexpansion

IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%CONFIGURATION%]==[] set CONFIGURATION=Release

ECHO Building release for %TARGET% (%TOOLSET%-%CONFIGURATION%)

del /q dist\%TARGET%\%TOOLSET%\%CONFIGURATION%

mkdir dist\%TARGET%\%TOOLSET%\%CONFIGURATION% > NUL
mkdir dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\lib > NUL
mkdir dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\include > NUL
mkdir dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin > NUL

set NATIVE_BUILD_PREFIX=dep\%TARGET%\%TOOLSET%\%CONFIGURATION%

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%CONFIGURATION%\lib
set SODIUM_STATIC=""
set CARGO_INCREMENTAL="1"
set RUST_BACKTRACE="1"
set RUST_FLAGS=""

if "%CONFIGURATION"=="Release" (
    set RUST_FLAGS="-C target-feature=+crt-static"
)

if "%CONFIGURATION"=="ReleaseDLL" (
    set RUST_FLAGS="-C target-feature=-crt-static"
)

ECHO Building dependencies for %TARGET% (%TOOLSET%-%CONFIGURATION%)

call dep.cmd || goto :error

call rustver.bat

rustup override set %RUST_VER%

ECHO Building safedrive CLI for %TARGET% (%TOOLSET%-%CONFIGURATION%)

cargo.exe build --release -p safedrive --target %TARGET% || goto :error

ECHO Building safedrive daemon for %TARGET% (%TOOLSET%-%CONFIGURATION%)

cargo.exe build --release -p safedrived --target %TARGET% || goto :error

ECHO Building SDDK headers for %TARGET% (%TOOLSET%-%CONFIGURATION%)

cheddar -f "libsafedrive\src\c_api.rs" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\include\sddk.h" || goto :error

ECHO Copying build artifacts for %TARGET% (%TOOLSET%-%CONFIGURATION%)

ECHO copying "target\%TARGET%\release\safedrive.dll" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\lib\safedrive.dll"
copy /y "target\%TARGET%\release\safedrive.dll" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\lib\safedrive.dll" || goto :error

ECHO copying "target\%TARGET%\release\safedrive.exe" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin\"
copy /y "target\%TARGET%\release\safedrive.exe" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin\" || goto :error

ECHO copying "target\%TARGET%\release\safedrived.exe" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin\"
copy /y "target\%TARGET%\release\safedrived.exe" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin\" || goto :error

goto :EOF

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!
