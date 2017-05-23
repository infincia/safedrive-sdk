@echo off

setlocal enabledelayedexpansion

IF [%ARCH%]==[] set ARCH=x64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%CONFIGURATION%]==[] set CONFIGURATION=Debug

set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[ReleaseDLL] set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[Release] set LIBSUFFIX=lib
IF [%CONFIGURATION%]==[DebugDLL] set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[Debug] set LIBSUFFIX=lib

ECHO Building release for %TARGET% (%TOOLSET%-%CONFIGURATION%)

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

set INTR_PREFIX=%cd%\build\SafeDriveSDK

set CMAKE_PREFIX=%cd%\Windows

del /q "!INTR_PREFIX!"

mkdir "!INTR_PREFIX!" > NUL

set OPENSSL_DIR=!BUILD_PREFIX!
set SODIUM_LIB_DIR=!BUILD_PREFIX!
set SODIUM_STATIC=""
set CARGO_INCREMENTAL="1"
set RUST_BACKTRACE="1"
set RUST_FLAGS=""

IF "!CONFIGURATION!"=="Release" (
    set RUNTIME_LIBRARY="MultiThreaded"
    set CMAKE_CXX_FLAGS_RELEASE="/MT"
    set CMAKE_C_FLAGS_RELEASE="/MT"
)

IF "!CONFIGURATION!"=="ReleaseDLL" (
    set RUNTIME_LIBRARY="MultiThreadedDLL"
    set CMAKE_CXX_FLAGS_RELEASE="/MD"
    set CMAKE_C_FLAGS_RELEASE="/MD"
)

IF "!CONFIGURATION!"=="Debug" (
    set RUNTIME_LIBRARY="MultiThreadedDebug"
    set CMAKE_CXX_FLAGS_DEBUG="/MTd"
    set CMAKE_C_FLAGS_DEBUG="/MTd"
)

IF "!CONFIGURATION!"=="DebugDLL" (
    set RUNTIME_LIBRARY="MultiThreadedDebugDLL"
    set CMAKE_CXX_FLAGS_DEBUG="/MDd"
    set CMAKE_C_FLAGS_DEBUG="/MDd"
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

cheddar -f "sddk\src\c_api.rs" "!BUILD_PREFIX!\include\sddk.h" || goto :error

ECHO Copying build artifacts for !TARGET! (!TOOLSET!-!CONFIGURATION!)

ECHO copying "target\!TARGET!\release\sddk.lib" "!BUILD_PREFIX!\"
copy /y "target\!TARGET!\release\sddk.lib" "!BUILD_PREFIX!\" || goto :error

ECHO copying "target\!TARGET!\release\safedrive.exe" "!BUILD_PREFIX!\safedrivecli.exe"
copy /y "target\!TARGET!\release\safedrive.exe" "!BUILD_PREFIX!\safedrivecli.exe" || goto :error

ECHO copying "target\!TARGET!\release\safedrived.exe" "!BUILD_PREFIX!\"
copy /y "target\!TARGET!\release\safedrived.exe" "!BUILD_PREFIX!\" || goto :error

pushd "!INTR_PREFIX!"
@echo building C++ SDK for "!VS!!CMAKE_GENERATOR_PLATFORM!"
cmake "!CMAKE_PREFIX!" -G"!VS!!CMAKE_GENERATOR_PLATFORM!" -T"!TOOLSET!" -D"TARGET=!TARGET!" -D"TOOLSET=!TOOLSET!" -D"CONFIGURATION=!CONFIGURATION!" -D"CMAKE_BUILD_TYPE=!CONFIGURATION!" -D"CMAKE_C_FLAGS_RELEASE=!CMAKE_C_FLAGS_RELEASE!" -D"CMAKE_CXX_FLAGS_RELEASE=!CMAKE_CXX_FLAGS_RELEASE!" -D"CMAKE_C_FLAGS_DEBUG=!CMAKE_C_FLAGS_DEBUG!" -D"CMAKE_CXX_FLAGS_DEBUG=!CMAKE_CXX_FLAGS_DEBUG!" || goto :error
msbuild /m /v:n /p:RuntimeLibrary=!RUNTIME_LIBRARY!;Configuration=!CONFIGURATION!;Platform=!PLATFORM!;PlatformToolset=!TOOLSET! SafeDriveSDK.sln || goto :error
popd
@echo copying "!INTR_PREFIX!\!CONFIGURATION!\SafeDriveSDK.!LIBSUFFIX!" to "!BUILD_PREFIX!\SafeDriveSDK.!LIBSUFFIX!"
copy /y "!INTR_PREFIX!\!CONFIGURATION!\SafeDriveSDK.!LIBSUFFIX!" "!BUILD_PREFIX!\SafeDriveSDK.!LIBSUFFIX!" || goto :error
goto :EOF

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:NORMALIZEPATH
  SET RETVAL=%~dpfn1
  EXIT /B

