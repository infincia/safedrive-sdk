ECHO testing SafeDrive for Windows-%BIT%

set SODIUM_LIB_DIR=%CD%\dep-%TARGET%-vs2015\lib
set SODIUM_STATIC=""

set SQLITE3_LIB_DIR=%CD%\dep-%TARGET%-vs2015\lib

pushd libsafedrive
cargo.exe test --release --verbose
popd