@echo off
setlocal
chcp 65001 >nul
title PowerBasilisk Enhanced Setup
echo ==================================================
echo   PowerBasilisk Enhanced - One-click Setup
echo ==================================================
echo.
echo This script checks and installs:
echo   1. Rust toolchain       (if missing)
echo   2. LLVM / Clang         (if missing, ~500 MB download)
echo   3. Windows SDK          (if missing)
echo   4. Builds pb_runtime_x64.obj and PB64Basic.exe
echo.
echo NOTE: If a Windows UAC prompt appears, click YES to
echo allow the installers to run.
echo.
pause
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup.ps1"
echo.
echo ==================================================
echo   Setup finished. You can close this window.
echo ==================================================
pause
