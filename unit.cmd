@echo off
setlocal

setlocal enabledelayedexpansion

set PLATFORM=%1
set CONFIGURATION=Debug
set TOOLSET=%3

IF "!PLATFORM!"=="x64" (
    set TARGET=x86_64-pc-windows-msvc
)

IF "!PLATFORM!"=="Win32" (
    set TARGET=i686-pc-windows-msvc
)

CALL :NORMALIZEPATH %cd%\!PLATFORM!\!CONFIGURATION!
SET BUILD_PREFIX=!RETVAL!

set OPENSSL_DIR=!BUILD_PREFIX!
set SODIUM_LIB_DIR=!BUILD_PREFIX!
set SODIUM_STATIC=""
set RUST_BACKTRACE=1

IF "!CONFIGURATION!"=="Release" (
    set BUILDOPTS=--release
)

IF "!CONFIGURATION!"=="Debug" (
    set BUILDOPTS=
)

call dep.cmd !PLATFORM! !CONFIGURATION! !TOOLSET! || goto :error

call rustver.bat

rustup override set !RUST_VER!

ECHO Testing sddk for !TARGET! (!PLATFORM!-!CONFIGURATION!-!TOOLSET!)

cargo.exe test !BUILDOPTS! -p sddk --target !TARGET! || goto :error
goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
endlocal
