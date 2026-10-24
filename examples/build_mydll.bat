@echo off
rem batch276: one-click build of the official MyDLL sample pair.
rem Builds batch276_MyDLL.dll + batch276_MyDLL.lib (default output names,
rem matching the DECLARE ... LIB "batch276_MyDLL.DLL" in
rem batch276_testDLL.bas), then links batch276_testDLL.exe.
rem Usage: run inside the examples folder, or from the repo root.
setlocal
cd /d "%~dp0"
set FC=..\target\release\PB64Basic.exe
if not exist "%FC%" set FC=target\release\PB64Basic.exe
if not exist "%FC%" (
    echo [ERROR] PB64Basic.exe not found. Build it first: cargo build --release -p pbcompiler
    exit /b 1
)
echo [1/2] Building batch276_MyDLL.dll + batch276_MyDLL.lib ...
"%FC%" build batch276_MyDLL.bas --target x86_64-pc-windows-msvc --runtime-lib ..\pb_runtime_x64.obj
if errorlevel 1 (
    echo [ERROR] DLL build failed.
    exit /b 1
)
echo [2/2] Linking batch276_testDLL.exe ...
"%FC%" build batch276_testDLL.bas --exe --target x86_64-pc-windows-msvc --runtime-lib ..\pb_runtime_x64.obj
if errorlevel 1 (
    echo [ERROR] TestDLL build failed.
    exit /b 1
)
echo OK - batch276_MyDLL.dll, batch276_MyDLL.lib, batch276_testDLL.exe written.
echo Run:  batch276_testDLL.exe (loads batch276_MyDLL.dll from its folder; MSGBOX pops up).
