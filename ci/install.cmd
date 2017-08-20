
setlocal enabledelayedexpansion

set PATH="C:\Users\%USER%\.cargo\bin;C:\Program Files\Git\bin;C:\Program Files\Git\mingw64\bin;C:\Program Files\7-Zip;C:\Program Files\WinAnt;%PATH%;"


set PLATFORM=%1

IF "!PLATFORM!"=="x64" (
    set TARGET=x86_64-pc-windows-msvc
    set ARCH=x64
    set VS="C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat"
    set TOOLSET=v141_xp
    set ESCRIPT=v141_xp64.bat
    set CONFIGURATION=Release
)

IF "!PLATFORM!"=="Win32" (
    set TARGET=i686-pc-windows-msvc
    set ARCH=x86
    set VS="C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat"
    set TOOLSET=v141_xp
    set ESCRIPT=v141_xp.bat
    set CONFIGURATION=Release
)

if defined VS call %VS% %ARCH%
if defined ESCRIPT call %ESCRIPT%
curl -sSf -o rustup-init.exe https://win.rustup.rs
rustup-init.exe --default-host %TARGET% -y
rustc -Vv
cargo -V
if not exist "C:\Users\%USER%\.cargo\bin\cheddar.exe" cargo install moz-cheddar

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
@echo finished install for !TARGET! (!PLATFORM!-!CONFIGURATION!-!TOOLSET!)
