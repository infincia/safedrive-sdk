@echo off

setlocal enabledelayedexpansion

set PATH=%USERPROFILE%\.cargo\bin;C:\Program Files\Git\bin;C:\Program Files\Git\mingw64\bin;C:\Program Files\7-Zip;C:\Program Files\WinAnt;%PATH%


set PLATFORM=%1
set CARGO_INCREMENTAL=1

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

rustup target add !TARGET! > NUL 2>&1

cargo run -p builder -- --toolset !TOOLSET! --platform !PLATFORM! --configuration !CONFIGURATION! test || goto :error

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
@echo main build finished for !TARGET! (!PLATFORM!-!CONFIGURATION!-!TOOLSET!)
