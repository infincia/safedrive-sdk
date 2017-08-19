@echo off

setlocal enabledelayedexpansion

set PLATFORM=%1
set CONFIGURATION=%2
set TOOLSET=%3

IF "!PLATFORM!"=="x64" (
    set TARGET=x86_64-pc-windows-msvc
    set ARCH=x64
    set VS="C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat"
    set TOOLSET=v141_xp
    set ESCRIPT=v141_xp64.bat
)

IF "!PLATFORM!"=="Win32" (
    set TARGET=i686-pc-windows-msvc
    set ARCH=x86
    set VS="C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat"
    set TOOLSET=v141_xp
    set ESCRIPT=v141_xp.bat
)

if defined VS call %VS% %ARCH%
if defined ESCRIPT call %ESCRIPT%
curl -sSf -o rustup-init.exe https://win.rustup.rs
rustup-init.exe --default-host %TARGET% -y
set PATH=C:\Users\%USER%\.cargo\bin;%PATH%;
rustc -Vv
cargo -V
if not exist "C:\Users\%USER%\.cargo\bin\cheddar.exe" cargo install moz-cheddar

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
@echo finished install for !TARGET! (!PLATFORM!-!CONFIGURATION!-!TOOLSET!)
