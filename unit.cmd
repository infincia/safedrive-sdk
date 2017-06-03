@echo off

setlocal enabledelayedexpansion

set PLATFORM=%1
set CONFIGURATION=%2
set TOOLSET=%3

IF "!PLATFORM!"=="x64" (
    set TARGET=x86_64-pc-windows-msvc
)

IF "!PLATFORM!"=="Win32" (
    set TARGET=i686-pc-windows-msvc
)

cargo run -p builder --target !TARGET! -- --toolset !TOOLSET! --platform !PLATFORM! --configuration !CONFIGURATION! test || goto :error

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
@echo main build finished for !TARGET! (!PLATFORM!-!CONFIGURATION!-!TOOLSET!)
