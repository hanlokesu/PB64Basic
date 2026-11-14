@echo off
rem compile.bat - one-click build for any example .bas (PB64Basic)
rem Usage: compile.bat [source.bas] [include-dir]
setlocal
set BAS=%~1
if "%BAS%"=="" set BAS=examples\hello.bas
set INCDIR=
if not "%~2"=="" set INCDIR=--include-dir "%~2"
PB64Basic.exe build "%BAS%" --exe --target x86_64-pc-windows-msvc --runtime-lib pb_runtime_x64.obj %INCDIR%
if errorlevel 1 (
  echo.
  echo COMPILE FAILED - see %BAS%.unimplemented.log if present
  pause
  exit /b 1
)
echo.
echo COMPILE OK - .exe written next to: %BAS%
pause
