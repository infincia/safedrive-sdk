@echo off

setlocal enabledelayedexpansion

set PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\Git\bin;C:\Program Files\Git\mingw64\bin;C:\Program Files\7-Zip;C:\Program Files\WinAnt;%PATH%


set PLATFORM=%1
set CARGO_INCREMENTAL=1

IF "!PLATFORM!"=="x64" (
    set ARCH=x64
    set VS=C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat
    set TOOLSET=v141_xp
    set ESCRIPT=v141_xp64.bat
    set CONFIGURATION=Release
)

IF "!PLATFORM!"=="Win32" (
    set ARCH=x86
    set VS=C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat
    set TOOLSET=v141_xp
    set ESCRIPT=v141_xp.bat
    set CONFIGURATION=Release
)

if defined VS call "%VS%" %ARCH%
if defined ESCRIPT call "%ESCRIPT%"
