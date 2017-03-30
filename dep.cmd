IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%LINKTYPE%]==[] set LINKTYPE=static

set LIBPREFIX=
IF [%TOOLSET%]==[v141_xp] set LIBPREFIX=lib

set LIBSUFFIX=dll
IF [%LINKTYPE%]==[dll] set LIBSUFFIX=dll
IF [%LINKTYPE%]==[static] set LIBSUFFIX=lib

set BUILD_PREFIX=%cd%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%


ECHO building libsodium for %TARGET% (%TOOLSET%-%LINKTYPE%)

del /q %BUILD_PREFIX%

mkdir %BUILD_PREFIX%
mkdir %BUILD_PREFIX%\lib
mkdir %BUILD_PREFIX%\include

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
    set PLATFORM=x86
)

set SODIUM_VER=1.0.12
set SODIUM_VER_FILE=%BUILD_PREFIX%\sodium_ver

set BUILD=false

findstr /c:"%SODIUM_VER%" %SODIUM_VER_FILE% > NUL
if errorlevel 1 set BUILD=true

IF "%BUILD%"=="true" (
    curl -L https://github.com/jedisct1/libsodium/releases/download/%SODIUM_VER%/libsodium-%SODIUM_VER%.tar.gz -o libsodium-%SODIUM_VER%.tar.gz
    7z x -y libsodium-%SODIUM_VER%.tar.gz
    7z x -y libsodium-%SODIUM_VER%.tar
    cd libsodium-%SODIUM_VER%
    msbuild /m /v:n /p:Configuration=%CONFIGURATION% /p:OutDir=%BUILD_PREFIX%\lib\;Platform=%PLATFORM%;PlatformToolset=%TOOLSET% libsodium.sln
    cd ..
    del /q libsodium-%SODIUM_VER%
    del /q libsodium-%SODIUM_VER%.tar
    del /q libsodium-%SODIUM_VER%.tar.gz
    copy /y %BUILD_PREFIX%\lib\%LIBPREFIX%sodium.%LIBSUFFIX% %BUILD_PREFIX%\lib\sodium.%LIBSUFFIX%
    @echo %SODIUM_VER%> %SODIUM_VER_FILE%
)