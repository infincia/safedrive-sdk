setlocal enabledelayedexpansion

IF [%ARCH%]==[] set ARCH=x86_64
IF [%TARGET%]==[] set TARGET=x86_64-pc-windows-msvc
IF [%TOOLSET%]==[] set TOOLSET=v141_xp
IF [%CONFIGURATION%]==[] set CONFIGURATION=Release

set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[ReleaseDLL] set LIBSUFFIX=dll
IF [%CONFIGURATION%]==[Release] set LIBSUFFIX=lib

ECHO Building release for %TARGET% (%TOOLSET%-%CONFIGURATION%)

del /q dist\%TARGET%\%TOOLSET%\%CONFIGURATION%

mkdir dist\%TARGET%\%TOOLSET%\%CONFIGURATION% > NUL
mkdir dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\lib > NUL
mkdir dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\include > NUL
mkdir dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin > NUL

set NATIVE_BUILD_PREFIX=%cd%\dep\%TARGET%\%TOOLSET%\%CONFIGURATION%

set OPENSSL_DIR=%NATIVE_BUILD_PREFIX%\lib
set SODIUM_LIB_DIR=%NATIVE_BUILD_PREFIX%\lib
set SODIUM_STATIC=""
set CARGO_INCREMENTAL="1"
set RUST_BACKTRACE="1"
set RUST_FLAGS=""

IF "%ARCH%"=="x86_64" (
    set PLATFORM=x64
    set CMAKE_GENERATOR_PLATFORM= Win64
)

IF "%ARCH%"=="x86" (
    set PLATFORM=Win32
    set CMAKE_GENERATOR_PLATFORM=
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

if "%CONFIGURATION"=="Release" (
    set RUST_FLAGS="-C target-feature=+crt-static"
)

if "%CONFIGURATION"=="ReleaseDLL" (
    set RUST_FLAGS="-C target-feature=-crt-static"
)

ECHO Building dependencies for %TARGET% (%TOOLSET%-%CONFIGURATION%)

call dep.cmd || goto :error

call rustver.bat

rustup override set %RUST_VER%

ECHO Building safedrive CLI for %TARGET% (%TOOLSET%-%CONFIGURATION%)

cargo.exe build --release -p safedrive --target %TARGET% || goto :error

ECHO Building safedrive daemon for %TARGET% (%TOOLSET%-%CONFIGURATION%)

cargo.exe build --release -p safedrived --target %TARGET% || goto :error

ECHO Building SDDK headers for %TARGET% (%TOOLSET%-%CONFIGURATION%)

cheddar -f "sddk\src\c_api.rs" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\include\sddk.h" || goto :error

ECHO Copying build artifacts for %TARGET% (%TOOLSET%-%CONFIGURATION%)

ECHO copying "target\%TARGET%\release\sddk.lib" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\lib\sddk.lib"
copy /y "target\%TARGET%\release\sddk.lib" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\lib\sddk.lib" || goto :error

ECHO copying "target\%TARGET%\release\safedrive.exe" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin\"
copy /y "target\%TARGET%\release\safedrive.exe" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin\" || goto :error

ECHO copying "target\%TARGET%\release\safedrived.exe" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin\"
copy /y "target\%TARGET%\release\safedrived.exe" "dist\%TARGET%\%TOOLSET%\%CONFIGURATION%\bin\" || goto :error

pushd Windows
@echo building C++ SDK for "!VS!!CMAKE_GENERATOR_PLATFORM!"

cmake . -G"!VS!!CMAKE_GENERATOR_PLATFORM!" -D"CMAKE_BUILD_TYPE=%CONFIGURATION%"
msbuild /m /v:n /p:OutDir="%BUILD_PREFIX%\lib\\";Configuration=%CONFIGURATION%;Platform=%PLATFORM%;PlatformToolset=%TOOLSET% SafeDriveSDK.sln || goto :error
popd
@echo copying "%BUILD_PREFIX%\lib\libSafeDriveSDK.%LIBSUFFIX%" to "%BUILD_PREFIX%\lib\SafeDriveSDK.%LIBSUFFIX%"
copy /y "%BUILD_PREFIX%\lib\libSafeDriveSDK.%LIBSUFFIX%" "%BUILD_PREFIX%\lib\SafeDriveSDK.%LIBSUFFIX%" || goto :error
goto :EOF

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!
