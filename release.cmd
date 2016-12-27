ECHO building SafeDrive for Windows-%BIT%

rmdir /s /q dist-win-%BIT%-vs2015
mkdir dist-win-%BIT%-vs2015
mkdir dist-win-%BIT%-vs2015\lib
mkdir dist-win-%BIT%-vs2015\include

set SODIUM_LIB_DIR=dep-win-%BIT%-vs2015\lib
set SODIUM_STATIC=""

set OPENSSL_DIR=%CD%\dep-win-%BIT%-vs2015
set OPENSSL_STATIC=""

set RUSTFLAGS=""

cargo.exe build --release --verbose

copy target\release\libsdsync.lib dist-win-%BIT%-vs2015\lib\

copy dep-win-%BIT%-vs2015\lib\* dist-win-%BIT%-vs2015\lib\
copy dep-win-%BIT%-vs2015\include\* dist-win-%BIT%-vs2015\include\
