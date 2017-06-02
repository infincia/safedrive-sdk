@echo off
setlocal

setlocal enabledelayedexpansion

set PLATFORM=%1
set CONFIGURATION=%2
set TOOLSET=%3

IF "!PLATFORM!"=="x64" (
    set TARGET=x86_64-pc-windows-msvc
)

IF "!PLATFORM!"=="Win32" (
    set TARGET=i686-pc-windows-msvc
    rustup target add !TARGET!
)

ECHO Building for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

CALL :NORMALIZEPATH %cd%\!PLATFORM!\!CONFIGURATION!
SET BUILD_PREFIX=!RETVAL!


CALL :NORMALIZEPATH %cd%\..\!CONFIGURATION!
SET DIST_PREFIX=!RETVAL!


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

ECHO Building safedrive CLI for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

cargo build !BUILDOPTS! --verbose -p safedrive --target !TARGET! || goto :error


ECHO Building SDDK headers for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

cheddar -f "sddk\src\c_api.rs" "!BUILD_PREFIX!\include\sddk.h" || goto :error

ECHO Copying build artifacts for !PLATFORM! (!CONFIGURATION!-!TOOLSET!)

ECHO copying "target\!TARGET!\!CONFIGURATION!\sddk.dll" "!BUILD_PREFIX!\"
copy /y "target\!TARGET!\!CONFIGURATION!\sddk.dll" "!BUILD_PREFIX!\" || goto :error

ECHO copying "target\!TARGET!\!CONFIGURATION!\safedrive.exe" "!DIST_PREFIX!\safedrivecli-!PLATFORM!.exe"
copy /y "target\!TARGET!\!CONFIGURATION!\safedrive.exe" "!DIST_PREFIX!\safedrivecli-!PLATFORM!.exe" || goto :error

endlocal

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:NORMALIZEPATH
  SET RETVAL=%~dpfn1
  EXIT /B

:done
endlocal