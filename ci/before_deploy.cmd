"C:\Program Files\Git\bin\git" describe > __query.tmp
set /p TAG=<__query.tmp
del __query.tmp

"C:\Program Files\7-Zip\7z" a safedrive-cli-%%TAG%%-%TARGET%.zip target\%TARGET%\safedrive
