ECHO "building %Platform%"

rmdir /s /q dist-win-%Platform%-vs2015

set SODIUM_LIB_DIR=dep-win-%Platform%-vs2015\lib
set SODIUM_STATIC=""

set OPENSSL_DIR=%CD%\dep-win-%Platform%-vs2015
set OPENSSL_STATIC=""

set RUSTFLAGS=""

cargo.exe build --release --verbose

mkdir dist-win-%Platform%-vs2015
mkdir dist-win-%Platform%-vs2015\lib
mkdir dist-win-%Platform%-vs2015\include

copy target\release\libsdsync.lib dist-win-%Platform%-vs2015\lib\

copy dep-win-%Platform%-vs2015\lib\* dist-win-%Platform%-vs2015\lib\
copy dep-win-%Platform%-vs2015\include\* dist-win-%Platform%-vs2015\include\
