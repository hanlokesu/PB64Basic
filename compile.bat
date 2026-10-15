@echo off
rem compile.bat - one-click build for any example .bas (PB64Basic)
setlocal
set BAS=%~1
if "%BAS%"=="" set BAS=examples\hello.bas
PB64Basic.exe build "%BAS%" --exe --target x86_64-pc-windows-msvc --runtime-lib pb_runtime_x64.obj
if errorlevel 1 (
  echo.
  echo COMPILE FAILED - see %BAS%.unimplemented.log if present
  pause
  exit /b 1
)
echo.
echo COMPILE OK - run: examples\hello.exe
pause
