"C:\Program Files\Git\bin\git" describe > __query.tmp
set /p TAG=<__query.tmp
del __query.tmp

7z a safedrive-sdk-%%TAG%%-%TARGET%.zip target\%TARGET%\safedrive
