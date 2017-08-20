
setlocal enabledelayedexpansion

set PATH="C:\Users\%USER%\.cargo\bin;C:\Program Files\Git\bin;C:\Program Files\Git\mingw64\bin;C:\Program Files\7-Zip;C:\Program Files\WinAnt;%PATH%;"

curl -sSf -o rustup-init.exe https://win.rustup.rs
rustup-init.exe --default-host %TARGET% -y
rustc -Vv
cargo -V
if not exist "C:\Users\%USER%\.cargo\bin\cheddar.exe" cargo install moz-cheddar

goto :done

:error
echo Failed with error #!errorlevel!.
exit /b !errorlevel!

:done
@echo finished install for !TARGET!
