ECHO testing SafeDrive for Windows-%BIT%

set SODIUM_LIB_DIR=%CD%\dep-win-%BIT%-vs2015\lib
set SODIUM_STATIC=""

set OPENSSL_DIR=%CD%\dep-win-%BIT%-vs2015
set OPENSSL_STATIC=""

set SQLITE3_LIB_DIR=%CD%\dep-win-%BIT%-vs2015\lib



set

cargo.exe test --release --verbose