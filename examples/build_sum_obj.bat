@echo off
rem batch260: one-click build of sum.obj from sum.c (clang / LLVM)
rem Usage: run inside the examples folder, or from the repo root with
rem        "build_sum_obj.bat" (paths below are relative to this file).
rem Requires clang on PATH (install LLVM, e.g. via winget install LLVM).
setlocal
cd /d "%~dp0"
echo Building sum.obj from sum.c ...
clang -c -O2 --target=x86_64-pc-windows-msvc sum.c -o sum.obj
if errorlevel 1 (
    echo [ERROR] clang failed. Is LLVM installed and on PATH?
    exit /b 1
)
echo OK - sum.obj written. Next: PB64Basic.exe build batch260_link_test.bas --exe --target x86_64-pc-windows-msvc --runtime-lib pb_runtime_x64.obj
