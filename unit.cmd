@echo off

setlocal enabledelayedexpansion

set PLATFORM=%1
set CONFIGURATION=%2
set TOOLSET=%3

IF "!PLATFORM!"=="x64" (
    set TARGET=x86_64-pc-windows-msvc
    set ARCH=x64
    set VS=C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat
    set TOOLSET=v141_xp
    set ESCRIPT=v141_xp64.bat
)

IF "!PLATFORM!"=="Win32" (
    set TARGET=i686-pc-windows-msvc
    set ARCH=x86
    set VS=C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat
    set TOOLSET=v141_xp
    set ESCRIPT=v141_xp.bat
)

if defined VS call "%VS%" %ARCH%
if defined ESCRIPT call "%ESCRIPT%"

cargo run -p builder --target !TARGET! -- --toolset !TOOLSET! --platform !PLATFORM! --configuration !CONFIGURATION! test || goto :error

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
@echo main build finished for !TARGET! (!PLATFORM!-!CONFIGURATION!-!TOOLSET!)
