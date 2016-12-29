ECHO building SafeDrive for Windows-%BIT%

mkdir dist-%TARGET%
mkdir dist-%TARGET%\lib
mkdir dist-%TARGET%\dep
mkdir dist-%TARGET%\bin

set SODIUM_LIB_DIR=%CD%\dep-%TARGET%-vs2015\lib
set SODIUM_STATIC=""

set SQLITE3_LIB_DIR=%CD%\dep-%TARGET%-vs2015\lib

pushd libsafedrive
cargo.exe build --release --verbose
popd
pushd safedrive
cargo.exe build --release --verbose
popd

robocopy %CD%\dep-%TARGET%-vs2015\ %CD%\dist-%TARGET%\dep\ /COPYALL /E

copy target\release\safedrive.lib dist-%TARGET%\lib\
copy target\release\safedrive.exe dist-%TARGET%\bin\

