@echo off

setlocal enabledelayedexpansion

IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%CONFIGURATION%]==[] set CONFIGURATION=Debug

set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[ReleaseDLL] set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[Release] set LIBSUFFIX=lib
IF [%CONFIGURATION%]==[DebugDLL] set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[Debug] set LIBSUFFIX=lib

ECHO Building release for %TARGET% (%TOOLSET%-%CONFIGURATION%)

set INTR_PREFIX=%cd%\build\%TARGET%\%TOOLSET%

set BUILD_PREFIX=%cd%\dep\%TARGET%\%TOOLSET%\%CONFIGURATION%
set DIST_PREFIX=%cd%\dist\%TARGET%\%TOOLSET%\%CONFIGURATION%
set CMAKE_PREFIX=%cd%\Windows

del /q "%DIST_PREFIX%"

del /q "%INTR_PREFIX%"

mkdir "%DIST_PREFIX%" > NUL
mkdir "%DIST_PREFIX%\lib" > NUL
mkdir "%DIST_PREFIX%\include" > NUL
mkdir "%DIST_PREFIX%\bin" > NUL

mkdir "%INTR_PREFIX%" > NUL

set OPENSSL_DIR=%BUILD_PREFIX%\lib
set SODIUM_LIB_DIR=%BUILD_PREFIX%\lib
set SODIUM_STATIC=""
set CARGO_INCREMENTAL="1"
set RUST_BACKTRACE="1"
set RUST_FLAGS=""

IF "!ARCH!"=="x86_64" (
    set PLATFORM=x64
    set CMAKE_GENERATOR_PLATFORM= Win64
)

IF "!ARCH!"=="x86" (
    set PLATFORM=Win32
    set CMAKE_GENERATOR_PLATFORM=
)

IF "!CONFIGURATION!"=="Release" (
    set RUNTIME_LIBRARY="MultiThreaded"
)

IF "!CONFIGURATION!"=="ReleaseDLL" (
    set RUNTIME_LIBRARY="MultiThreadedDLL"
)

IF "!CONFIGURATION!"=="Debug" (
    set RUNTIME_LIBRARY="MultiThreadedDebug"
)

IF "!CONFIGURATION!"=="DebugDLL" (
    set RUNTIME_LIBRARY="MultiThreadedDebugDLL"
)

IF "!TOOLSET!"=="v120_xp" (
    set VS=Visual Studio 12 2013
)

IF "!TOOLSET!"=="v140_xp" (
    set VS=Visual Studio 14 2015
)

IF "!TOOLSET!"=="v141_xp" (
    set VS=Visual Studio 15 2017
)

if "!CONFIGURATION!"=="Release" (
    set RUST_FLAGS="-C target-feature=+crt-static"
)

if "!CONFIGURATION!"=="ReleaseDLL" (
    set RUST_FLAGS="-C target-feature=-crt-static"
)

if "!CONFIGURATION!"=="Debug" (
    set RUST_FLAGS="-C target-feature=+crt-static"
)

if "!CONFIGURATION!"=="DebugDLL" (
    set RUST_FLAGS="-C target-feature=-crt-static"
)

ECHO Building dependencies for !TARGET! (!TOOLSET!-!CONFIGURATION!)

call dep.cmd || goto :error

call rustver.bat

rustup override set !RUST_VER!

ECHO Building safedrive CLI for !TARGET! (!TOOLSET!-!CONFIGURATION!)

cargo.exe build --release --verbose -p safedrive --target !TARGET! || goto :error

ECHO Building safedrive daemon for !TARGET! (!TOOLSET!-!CONFIGURATION!)

cargo.exe build --release --verbose -p safedrived --target !TARGET! || goto :error

ECHO Building SDDK headers for !TARGET! (!TOOLSET!-!CONFIGURATION!)

cheddar -f "sddk\src\c_api.rs" "!DIST_PREFIX!\include\sddk.h" || goto :error

ECHO Copying build artifacts for !TARGET! (!TOOLSET!-!CONFIGURATION!)

ECHO copying "target\!TARGET!\release\sddk.lib" "!DIST_PREFIX!\lib\"
copy /y "target\!TARGET!\release\sddk.lib" "!DIST_PREFIX!\lib\" || goto :error

ECHO copying "target\!TARGET!\release\safedrive.exe" "!DIST_PREFIX!\bin\"
copy /y "target\!TARGET!\release\safedrive.exe" "!DIST_PREFIX!\bin\" || goto :error

ECHO copying "target\!TARGET!\release\safedrived.exe" "!DIST_PREFIX!\bin\"
copy /y "target\!TARGET!\release\safedrived.exe" "!DIST_PREFIX!\bin\" || goto :error

pushd "!INTR_PREFIX!"
@echo building C++ SDK for "!VS!!CMAKE_GENERATOR_PLATFORM!"
cmake "!CMAKE_PREFIX!" -G"!VS!!CMAKE_GENERATOR_PLATFORM!" -T"!TOOLSET!" -D"TARGET=!TARGET!" -D"TOOLSET=!TOOLSET!" -D"CONFIGURATION=!CONFIGURATION!" -D"CMAKE_BUILD_TYPE=!CONFIGURATION!" || goto :error
msbuild /m /v:n /p:RuntimeLibrary=!RUNTIME_LIBRARY!;Configuration=!CONFIGURATION!;Platform=!PLATFORM!;PlatformToolset=!TOOLSET! SafeDriveSDK.sln || goto :error
popd
@echo copying "!INTR_PREFIX!\!CONFIGURATION!\SafeDriveSDK.!LIBSUFFIX!" to "!DIST_PREFIX!\lib\SafeDriveSDK.!LIBSUFFIX!"
copy /y "!INTR_PREFIX!\!CONFIGURATION!\SafeDriveSDK.!LIBSUFFIX!" "!DIST_PREFIX!\lib\SafeDriveSDK.!LIBSUFFIX!" || goto :error
goto :EOF

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!
