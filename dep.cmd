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
mkdir "%BUILD_PREFIX%\include\openssl" > NUL

mkdir "%SRC_PREFIX%" > NUL
mkdir build > NUL

IF "%ARCH%"=="x86_64" (
    set PLATFORM=x64
    set CMAKE_GENERATOR_PLATFORM= Win64
)

IF "%ARCH%"=="x86" (
    set PLATFORM=Win32
    set CMAKE_GENERATOR_PLATFORM=
)

IF "!CONFIGURATION!"=="Release" (
    set RUNTIME_LIBRARY="MultiThreaded"
)

IF "!CONFIGURATION!"=="ReleaseDLL" (
    set RUNTIME_LIBRARY="MultiThreadedDLL"
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

set LIBRESSL_VER=2.5.3
set LIBRESSL_VER_FILE="%BUILD_PREFIX%\libressl_ver"

pushd "%SRC_PREFIX%"

IF NOT EXIST libsodium-%SODIUM_VER%.tar.gz (
    @echo downloading libsodium
    curl -L https://github.com/jedisct1/libsodium/releases/download/%SODIUM_VER%/libsodium-%SODIUM_VER%.tar.gz -o libsodium-%SODIUM_VER%.tar.gz
)

IF NOT EXIST libssh2-%LIBSSH2_VER%.tar.gz (
    @echo downloading libssh2
    curl -L https://www.libssh2.org/download/libssh2-%LIBSSH2_VER%.tar.gz -o libssh2-%LIBSSH2_VER%.tar.gz
)

IF NOT EXIST libressl-%LIBRESSL_VER%.tar.gz (
    @echo downloading libressl
    curl -L https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-%LIBRESSL_VER%.tar.gz -o libressl-%LIBRESSL_VER%.tar.gz
)

popd

IF NOT EXIST "%SRC_PREFIX%\libsodium-%SODIUM_VER%.tar.gz" goto :error
IF NOT EXIST "%SRC_PREFIX%\libssh2-%LIBSSH2_VER%.tar.gz" goto :error
IF NOT EXIST "%SRC_PREFIX%\libressl-%LIBRESSL_VER%.tar.gz" goto :error


:checksodium

IF NOT EXIST "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%" goto :buildsodium

findstr /c:"%SODIUM_VER%" %SODIUM_VER_FILE% > NUL || goto :buildsodium
goto :EOF



:buildsodium

pushd build
@echo unpacking libsodium source
del /q libsodium-%SODIUM_VER%
7z x -y "%SRC_PREFIX%\libsodium-%SODIUM_VER%.tar.gz" || goto :error
7z x -y "libsodium-%SODIUM_VER%.tar" || goto :error
pushd libsodium-%SODIUM_VER%
@echo building libsodium
msbuild /m /v:n /p:OutDir="%BUILD_PREFIX%\lib\\";RuntimeLibrary=!RUNTIME_LIBRARY!;Configuration=!CONFIGURATION!;Platform=!PLATFORM!;PlatformToolset=!TOOLSET! libsodium.sln || goto :error
popd
del /q libsodium-%SODIUM_VER%
@echo copying "%BUILD_PREFIX%\lib\libsodium.%LIBSUFFIX%" to "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%"
copy /y "%BUILD_PREFIX%\lib\libsodium.%LIBSUFFIX%" "%BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%" || goto :error
@echo %SODIUM_VER%> %SODIUM_VER_FILE%
popd
goto :EOF



:checklibressl

IF NOT EXIST "%BUILD_PREFIX%\lib\ssl.%LIBSUFFIX%" goto :buildlibressl

findstr /c:"%LIBRESSL_VER%" %LIBRESSL_VER_FILE% > NUL || goto :buildlibressl
goto :checkssh2

:buildlibressl

pushd build
@echo unpacking libressl source
del /q libressl-%LIBRESSL_VER%
7z x -y "%SRC_PREFIX%\libressl-%LIBRESSL_VER%.tar.gz" || goto :error
7z x -y "libressl-%LIBRESSL_VER%.tar" || goto :error
pushd libressl-%LIBRESSL_VER%
@echo building libressl for "!VS!!CMAKE_GENERATOR_PLATFORM!"
cmake . -G"!VS!!CMAKE_GENERATOR_PLATFORM!" -T"!TOOLSET!"  -D"BUILD_SHARED_LIBS=0" -D"BUILD_EXAMPLES=0" -D"BUILD_TESTING=0" -D"CMAKE_BUILD_TYPE=!CONFIGURATION!"
msbuild /m /v:n /p:RuntimeLibrary=!RUNTIME_LIBRARY!;Configuration=!CONFIGURATION!;Platform=!PLATFORM!;PlatformToolset=!TOOLSET! libressl.sln || goto :error
@echo copying "include\openssl" to "%BUILD_PREFIX%\include\"
copy /y "include\openssl"  "%BUILD_PREFIX%\include\openssl\" || goto :error
@echo copying "ssl\%CONFIGURATION%\ssl.%LIBSUFFIX%" to "%BUILD_PREFIX%\lib\ssl.%LIBSUFFIX%"
copy /y "ssl\%CONFIGURATION%\ssl.%LIBSUFFIX%" "%BUILD_PREFIX%\lib\ssl.%LIBSUFFIX%" || goto :error
@echo copying "tls\%CONFIGURATION%\tls.%LIBSUFFIX%" to "%BUILD_PREFIX%\lib\tls.%LIBSUFFIX%"
copy /y "tls\%CONFIGURATION%\tls.%LIBSUFFIX%" "%BUILD_PREFIX%\lib\tls.%LIBSUFFIX%" || goto :error
@echo copying "crypto\%CONFIGURATION%\crypto.%LIBSUFFIX%" to "%BUILD_PREFIX%\lib\crypto.%LIBSUFFIX%"
copy /y "crypto\%CONFIGURATION%\crypto.%LIBSUFFIX%" "%BUILD_PREFIX%\lib\crypto.%LIBSUFFIX%" || goto :error
@echo %LIBRESSL_VER%> %LIBRESSL_VER_FILE%
popd
del /q libressl-%LIBRESSL_VER%
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
@echo building libssh2 for "!VS!!CMAKE_GENERATOR_PLATFORM!"
cmake . -G"!VS!!CMAKE_GENERATOR_PLATFORM!" -T"%TOOLSET%" -D"BUILD_SHARED_LIBS=0" -D"BUILD_EXAMPLES=0" -D"BUILD_TESTING=0" -D"CMAKE_BUILD_TYPE=!CONFIGURATION!" -D"OPENSSL_USE_STATIC_LIBS=TRUE" -D"CRYPTO_BACKEND=OpenSSL" -D"OPENSSL_ROOT_DIR="%BUILD_PREFIX%\\" -D"OPENSSL_INCLUDE_DIR="%BUILD_PREFIX%\include\\"
msbuild /m /v:n /p:OutDir="%BUILD_PREFIX%\lib\\";RuntimeLibrary=!RUNTIME_LIBRARY!;Configuration=!CONFIGURATION!;Platform=!PLATFORM!;PlatformToolset=!TOOLSET! libssh2.sln || goto :error
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
