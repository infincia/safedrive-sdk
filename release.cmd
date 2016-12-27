ECHO building SafeDrive for Windows-%BIT%

mkdir dist-win-%BIT%-vs2015
mkdir dist-win-%BIT%-vs2015\lib
mkdir dist-win-%BIT%-vs2015\include

set SODIUM_LIB_DIR=%CD%\dep-win-%BIT%-vs2015\lib
set SODIUM_STATIC=""

set OPENSSL_DIR=%CD%\dep-win-%BIT%-vs2015
set OPENSSL_STATIC=""

if [%BIT%] EQU [x64] (
    echo linking 64bit sqlite
    set SQLITE3_LIB_DIR=C:\Users\appveyor\lib64
)

if [%BIT%] EQU [x86] (
    echo linking 32bit sqlite
    set SQLITE3_LIB_DIR=C:\Users\appveyor\lib
)


set

cargo.exe build --release --verbose

copy target\release\libsdsync.lib dist-win-%BIT%-vs2015\lib\

copy dep-win-%BIT%-vs2015\lib\* dist-win-%BIT%-vs2015\lib\
copy dep-win-%BIT%-vs2015\include\* dist-win-%BIT%-vs2015\include\
