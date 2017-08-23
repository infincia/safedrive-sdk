@echo off

setlocal enabledelayedexpansion

set PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\Git\bin;C:\Program Files\Git\mingw64\bin;C:\Program Files\7-Zip;C:\Program Files\WinAnt;%PATH%


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

cargo run -p builder --target !TARGET! -- --toolset !TOOLSET! --platform !PLATFORM! --configuration !CONFIGURATION! test || goto :error

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
@echo main build finished for !TARGET! (!PLATFORM!-!CONFIGURATION!-!TOOLSET!)
