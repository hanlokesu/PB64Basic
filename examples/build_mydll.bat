@echo off
rem batch276: one-click build of the official MyDLL sample pair.
rem Builds MyDLL.dll + MyDLL.lib (via -o MyDLL.obj, so the output names
rem match the DECLARE ... LIB "MYDLL.DLL" in batch276_testDLL.bas), then
rem links batch276_testDLL.exe against MYDLL.lib.
rem Usage: run inside the examples folder, or from the repo root.
setlocal
cd /d "%~dp0"
set FC=..\target\release\PB64Basic.exe
if not exist "%FC%" set FC=target\release\PB64Basic.exe
if not exist "%FC%" (
    echo [ERROR] PB64Basic.exe not found. Build it first: cargo build --release -p pbcompiler
    exit /b 1
)
echo [1/2] Building MyDLL.dll + MyDLL.lib ...
"%FC%" build batch276_MyDLL.bas -o MyDLL.obj --target x86_64-pc-windows-msvc --runtime-lib ..\pb_runtime_x64.obj
if errorlevel 1 (
    echo [ERROR] DLL build failed.
    exit /b 1
)
echo [2/2] Linking batch276_testDLL.exe against MYDLL.lib ...
"%FC%" build batch276_testDLL.bas --exe --target x86_64-pc-windows-msvc --runtime-lib ..\pb_runtime_x64.obj
if errorlevel 1 (
    echo [ERROR] TestDLL build failed.
    exit /b 1
)
echo OK - MyDLL.dll, MyDLL.lib, batch276_testDLL.exe written.
echo Run:  copy MyDLL.dll next to batch276_testDLL.exe, then run it (MSGBOX dialog pops up).
