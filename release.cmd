ECHO building SafeDrive for Windows-%ARCH%

mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\include
mkdir dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin

set SODIUM_LIB_DIR=%CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\lib
set SODIUM_STATIC=""

IF "%LINKTYPE%"=="mt" (
    set RUSTFLAGS=-Z unstable-options -C target-feature=+crt-static
)

rustup default %CHANNEL%-%TARGET%

pushd libsafedrive
cargo.exe build --release --verbose
popd
pushd safedrive
cargo.exe build --release --verbose
popd

robocopy %CD%\dep\%TARGET%\%TOOLSET%\%LINKTYPE%\ %CD%\dist-%TARGET%-%TOOLSET%-%LINKTYPE%\ /COPYALL /E

copy /y target\release\deps\safedrive*.dll.lib dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll.lib
copy /y target\release\deps\safedrive*.dll.exp dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll.exp
copy /y target\release\deps\safedrive*.dll dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.dll
copy /y target\release\deps\safedrive*.pdb dist-%TARGET%-%TOOLSET%-%LINKTYPE%\lib\safedrive.pdb

copy /y target\release\safedrive.exe dist-%TARGET%-%TOOLSET%-%LINKTYPE%\bin\

