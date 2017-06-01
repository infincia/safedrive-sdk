@echo off

setlocal enabledelayedexpansion

IF [%ARCH%]==[] set ARCH=x64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%CONFIGURATION%]==[] set CONFIGURATION=Debug


ECHO Building release for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

IF "!ARCH!"=="x64" (
    set PLATFORM=x64
    set CMAKE_GENERATOR_PLATFORM= Win64
)

IF "!ARCH!"=="x86" (
    set PLATFORM=Win32
    set CMAKE_GENERATOR_PLATFORM=
)

CALL :NORMALIZEPATH %cd%\..\!PLATFORM!\!CONFIGURATION!
SET BUILD_PREFIX=%RETVAL%




set OPENSSL_DIR=!BUILD_PREFIX!
set SODIUM_LIB_DIR=!BUILD_PREFIX!
set SODIUM_STATIC=""
set CARGO_INCREMENTAL="1"
set RUST_BACKTRACE="1"
set RUSTFLAGS=""

IF "!CONFIGURATION!"=="Release" (
    set RUNTIME_LIBRARY="MultiThreadedDLL"
)

IF "!CONFIGURATION!"=="Debug" (
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

ECHO Building dependencies for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

call dep.cmd || goto :error

call rustver.bat

rustup override set !RUST_VER!

ECHO Building safedrive CLI for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

cargo.exe build --release --verbose -p safedrive --target !TARGET! || goto :error

ECHO Building safedrive daemon for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

cargo.exe build --release --verbose -p safedrived --target !TARGET! || goto :error

ECHO Building SDDK headers for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

cheddar -f "sddk\src\c_api.rs" "!BUILD_PREFIX!\include\sddk.h" || goto :error

ECHO Copying build artifacts for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

ECHO copying "target\!TARGET!\release\sddk.lib" "!BUILD_PREFIX!\"
copy /y "target\!TARGET!\release\sddk.lib" "!BUILD_PREFIX!\" || goto :error

ECHO copying "target\!TARGET!\release\sddk.dll" "!BUILD_PREFIX!\"
copy /y "target\!TARGET!\release\sddk.dll" "!BUILD_PREFIX!\" || goto :error

ECHO copying "target\!TARGET!\release\safedrive.exe" "!BUILD_PREFIX!\safedrivecli.exe"
copy /y "target\!TARGET!\release\safedrive.exe" "!BUILD_PREFIX!\safedrivecli.exe" || goto :error

ECHO copying "target\!TARGET!\release\safedrived.exe" "!BUILD_PREFIX!\"
copy /y "target\!TARGET!\release\safedrived.exe" "!BUILD_PREFIX!\" || goto :error



goto :EOF

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:NORMALIZEPATH
  SET RETVAL=%~dpfn1
  EXIT /B

