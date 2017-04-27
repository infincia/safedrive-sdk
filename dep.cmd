setlocal enabledelayedexpansion

IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%CONFIGURATION%]==[] set CONFIGURATION=Release

set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[ReleaseDLL] set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[Release] set LIBSUFFIX=lib

set BUILD_PREFIX=%cd%\dep\%TARGET%\%TOOLSET%\%CONFIGURATION%
set SRC_PREFIX=%cd%\src

ECHO building dependencies for %TARGET% (%TOOLSET%-%CONFIGURATION%)

mkdir "%BUILD_PREFIX%" > NUL
mkdir "%BUILD_PREFIX%\lib" > NUL
mkdir "%BUILD_PREFIX%\include" > NUL

mkdir "%SRC_PREFIX%" > NUL
mkdir build > NUL

IF "%ARCH%"=="x86_64" (
    set PLATFORM=x64
)

IF "%ARCH%"=="x86" (
    set PLATFORM=Win32
)

IF "%TOOLSET%"=="v120_xp" (
    set VS=Visual Studio 12 2013
)

IF "%TOOLSET%"=="v140_xp" (
    set VS=Visual Studio 14 2015
)

IF "%TOOLSET%"=="v141_xp" (
    set VS=Visual Studio 15 2017
)

set SODIUM_VER=1.0.12
set SODIUM_VER_FILE="%BUILD_PREFIX%\sodium_ver"

set LIBSSH2_VER=1.8.0
set LIBSSH2_VER_FILE="%BUILD_PREFIX%\ssh2_ver"

pushd "%SRC_PREFIX%"

IF NOT EXIST libsodium-%SODIUM_VER%.tar.gz (
    @echo downloading libsodium
    curl -L https://github.com/jedisct1/libsodium/releases/download/%SODIUM_VER%/libsodium-%SODIUM_VER%.tar.gz -o libsodium-%SODIUM_VER%.tar.gz
)

IF NOT EXIST libssh2-%LIBSSH2_VER%.tar.gz (
    @echo downloading libssh2
    curl -L https://www.libssh2.org/download/libssh2-%LIBSSH2_VER%.tar.gz -o libssh2-%LIBSSH2_VER%.tar.gz
)

popd

IF NOT EXIST "%SRC_PREFIX%\libsodium-%SODIUM_VER%.tar.gz" goto :error
IF NOT EXIST "%SRC_PREFIX%\libssh2-%LIBSSH2_VER%.tar.gz" goto :error


:checksodium

IF NOT EXIST "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%" goto :buildsodium

findstr /c:"%SODIUM_VER%" %SODIUM_VER_FILE% > NUL || goto :buildsodium
goto :checkssh2



:buildsodium

pushd build
@echo unpacking libsodium source
del /q libsodium-%SODIUM_VER%
7z x -y "%SRC_PREFIX%\libsodium-%SODIUM_VER%.tar.gz" || goto :error
7z x -y "libsodium-%SODIUM_VER%.tar" || goto :error
pushd libsodium-%SODIUM_VER%
@echo building libsodium
msbuild /m /v:n /p:OutDir="%BUILD_PREFIX%\lib\\";Configuration=%CONFIGURATION%;Platform=%PLATFORM%;PlatformToolset=%TOOLSET% libsodium.sln || goto :error
popd
del /q libsodium-%SODIUM_VER%
@echo copying "%BUILD_PREFIX%\lib\libsodium.%LIBSUFFIX%" to "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%"
copy /y "%BUILD_PREFIX%\lib\libsodium.%LIBSUFFIX%" "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%" || goto :error
@echo %SODIUM_VER%> %SODIUM_VER_FILE%
popd
goto :checkssh2



:checkssh2

IF NOT EXIST "%BUILD_PREFIX%\lib\ssh2.%LIBSUFFIX%" goto :buildssh2

findstr /c:"%LIBSSH2_VER%" %LIBSSH2_VER_FILE% > NUL || goto :buildssh2
goto :EOF

:buildssh2

pushd build
@echo unpacking libssh2 source
del /q libssh2-%LIBSSH2_VER%
7z x -y "%SRC_PREFIX%\libssh2-%LIBSSH2_VER%.tar.gz" || goto :error
7z x -y "libssh2-%LIBSSH2_VER%.tar" || goto :error
pushd libssh2-%LIBSSH2_VER%
@echo building libssh2
cmake . -G"!VS! Win64" -D"BUILD_SHARED_LIBS=0" -D"BUILD_EXAMPLES=0" -D"BUILD_TESTING=0" -D"CMAKE_BUILD_TYPE=Release" -D"CRYPTO_BACKEND=WinCNG"
msbuild /m /v:n /p:OutDir="%BUILD_PREFIX%\lib\\";Configuration=%CONFIGURATION%;Platform=%PLATFORM%;PlatformToolset=%TOOLSET% libssh2.sln || goto :error
popd
del /q libssh2-%LIBSSH2_VER%
@echo copying "%BUILD_PREFIX%\lib\libssh2.%LIBSUFFIX%" to "%BUILD_PREFIX%\lib\ssh2.%LIBSUFFIX%"
copy /y "%BUILD_PREFIX%\lib\libssh2.%LIBSUFFIX%" "%BUILD_PREFIX%\lib\ssh2.%LIBSUFFIX%" || goto :error
@echo %LIBSSH2_VER%> %LIBSSH2_VER_FILE%
popd
goto :EOF

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!
