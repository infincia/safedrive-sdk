ECHO building SafeDrive for Windows-%BIT%

mkdir dist
mkdir dist\lib
mkdir dist\dep
mkdir dist\bin

set SODIUM_LIB_DIR=%CD%\dep-win-%BIT%-vs2015\lib
set SODIUM_STATIC=""

set SQLITE3_LIB_DIR=%CD%\dep-win-%BIT%-vs2015\lib

pushd libsafedrive
cargo.exe build --release --verbose
popd
pushd safedrive
cargo.exe build --release --verbose
popd

robocopy %CD%\dep-win-%BIT%-vs2015\ %CD%\dist\dep\ /COPYALL /E

copy target\release\safedrive.lib dist\lib\
copy target\release\safedrive.exe dist\bin\

