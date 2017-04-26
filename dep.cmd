setlocal enabledelayedexpansion

IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%LINKTYPE%]==[] set LINKTYPE=static

set LIBSUFFIX=dll
IF [%LINKTYPE%]==[dll] set LIBSUFFIX=dll
IF [%LINKTYPE%]==[static] set LIBSUFFIX=lib

set BUILD_PREFIX=%cd%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%
set SRC_PREFIX=%cd%\src

ECHO building libsodium for %TARGET% (%TOOLSET%-%LINKTYPE%)

mkdir "%BUILD_PREFIX%"
mkdir "%BUILD_PREFIX%\lib"
mkdir "%BUILD_PREFIX%\include"

mkdir "%SRC_PREFIX%"
mkdir build

IF "%LINKTYPE%"=="static" (
    set CONFIGURATION=Release
)

IF "%LINKTYPE%"=="dll" (
    set CONFIGURATION=ReleaseDLL
)

IF "%ARCH%"=="x86_64" (
    set PLATFORM=x64
)

IF "%ARCH%"=="x86" (
    set PLATFORM=Win32
)

set SODIUM_VER=1.0.12
set SODIUM_VER_FILE="%BUILD_PREFIX%\sodium_ver"

pushd "%SRC_PREFIX%"

IF NOT EXIST libsodium-%SODIUM_VER%.tar.gz (
    curl -L https://github.com/jedisct1/libsodium/releases/download/%SODIUM_VER%/libsodium-%SODIUM_VER%.tar.gz -o libsodium-%SODIUM_VER%.tar.gz || goto :error
)

popd

IF NOT EXIST "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%" || goto :build
goto :EOF

findstr /c:"%SODIUM_VER%" %SODIUM_VER_FILE% > NUL || goto :build
goto :EOF



:build

pushd build
@echo unpacking source
del /q libsodium-%SODIUM_VER%
7z x -y "%SRC_PREFIX%\libsodium-%SODIUM_VER%.tar.gz" || goto :error
7z x -y "%SRC_PREFIX%\libsodium-%SODIUM_VER%.tar" || goto :error
pushd libsodium-%SODIUM_VER%
@echo building
msbuild /m /v:n /p:OutDir="%BUILD_PREFIX%\lib\\";Configuration=%CONFIGURATION%;Platform=%PLATFORM%;PlatformToolset=%TOOLSET% libsodium.sln || goto :error
popd
del /q libsodium-%SODIUM_VER%
@echo copying "%BUILD_PREFIX%\lib\libsodium.%LIBSUFFIX%" to "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%"
copy /y "%BUILD_PREFIX%\lib\libsodium.%LIBSUFFIX%" "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%" || goto :error
@echo %SODIUM_VER%> %SODIUM_VER_FILE%
popd
goto :EOF


:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!
