ECHO testing SafeDrive for Windows-%BIT%

set SODIUM_LIB_DIR=%CD%\dep-win-%BIT%-vs2015\lib
set SODIUM_STATIC=""

set OPENSSL_DIR=%CD%\dep-win-%BIT%-vs2015
set OPENSSL_STATIC=""

if [%BIT%] EQU [x64] (
    echo linking 64bit sqlite
    set SQLITE3_LIB_DIR=C:\Users\appveyor\libx64
)

if [%BIT%] EQU [x86] (
    echo linking 32bit sqlite
    set SQLITE3_LIB_DIR=C:\Users\appveyor\libx86
)


set

cargo.exe test --release --verbose