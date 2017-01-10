ECHO building SafeDrive for Windows-%ARCH%

mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\dep
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin

set SODIUM_LIB_DIR=%CD%\dep-%TARGET%-%TOOLSET%-%LINKTYPE%\lib

IF "%LINKTYPE%"=="mt" (
    set SODIUM_STATIC=""
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
)

rustup default %CHANNEL%-%TARGET%

pushd libsafedrive
cargo.exe build --release --verbose
popd
pushd safedrive
cargo.exe build --release --verbose
popd

robocopy %CD%\dep-%TARGET%-%TOOLSET%-%LINKTYPE%\ %CD%\dist-%TARGET%-%TOOLSET%-%LINKTYPE%\dep\ /COPYALL /E

copy /y target\release\deps\safedrive*.lib dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.lib
copy /y target\release\safedrive.exe dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\

