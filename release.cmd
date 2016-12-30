ECHO building SafeDrive for Windows-%ARCH%

mkdir dist-%TARGET%-%TOOLSET%
mkdir dist-%TARGET%-%TOOLSET%\lib
mkdir dist-%TARGET%-%TOOLSET%\include
mkdir dist-%TARGET%-%TOOLSET%\dep
mkdir dist-%TARGET%-%TOOLSET%\bin

set SODIUM_LIB_DIR=%CD%\dep-%TARGET%-%TOOLSET%\lib
set SODIUM_STATIC=""

set SQLITE3_LIB_DIR=%CD%\dep-%TARGET%-%TOOLSET%\lib

set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static

pushd libsafedrive
cargo.exe build --release --verbose
popd
pushd safedrive
cargo.exe build --release --verbose
popd

robocopy %CD%\dep-%TARGET%-%TOOLSET%\ %CD%\dist-%TARGET%-%TOOLSET%\dep\ /COPYALL /E

copy target\release\deps\safedrive.lib dist-%TARGET%-%TOOLSET%\lib\
copy target\release\safedrive.exe dist-%TARGET%-%TOOLSET%\bin\

