@echo off

setlocal enabledelayedexpansion

set PLATFORM=%1
set CONFIGURATION=%2
set TOOLSET=%3
set CARGO_INCREMENTAL=1


IF "!PLATFORM!"=="x64" (
    set TARGET=x86_64-pc-windows-msvc
)

IF "!PLATFORM!"=="Win32" (
    set TARGET=i686-pc-windows-msvc
)

rustup target add !TARGET! > NUL 2>&1

cargo run -p builder -- --toolset !TOOLSET! --platform !PLATFORM! --configuration !CONFIGURATION! build || goto :error

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
@echo main build finished for !TARGET! (!PLATFORM!-!CONFIGURATION!-!TOOLSET!)
