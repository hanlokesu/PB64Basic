@echo off
setlocal
chcp 65001 >nul
title PB64Basic Setup
echo ==================================================
echo   PB64Basic - One-click Setup
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
echo.
echo ==================================================
echo   
echo.
echo ==================================================
echo   
echo.
echo ==================================================
echo   
echo.
echo ==================================================
echo   Optional: JK-IDE integration (Compilers menu)
echo ==================================================
echo This installs the PB64Basic Compilers menu into JK-IDE so you can
echo compile the active file with PBWin 10 / PB64Basic / PBXB64.
echo Type Y to install, anything else to skip.
set /p "JKDO=Install JK-IDE integration? (Y/N): "
if /I not "%JKDO%"=="Y" goto :skip_jk
set "JKIDE_DIR="
if exist "%ProgramFiles(x86)%\JK-IDE\jk-ide.exe" set "JKIDE_DIR=%ProgramFiles(x86)%\JK-IDE"
if exist "C:\PBWin10\JK-IDE\jk-ide.exe" set "JKIDE_DIR=C:\PBWin10\JK-IDE"
if not defined JKIDE_DIR goto :jk_dl
goto :jkdir_ok
:jk_dl
set /p "JKDO2=JK-IDE not found. Auto-download it from the author official site (~43 MB)? (Y/N): "
if /I not "%JKDO2%"=="Y" goto :jkdir_manual
echo Downloading jkb.zip from the author official Dropbox (Juergen Kuehlwein)...
set "JKDL=%TEMP%\jkb_download.zip"
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ProgressPreference='SilentlyContinue'; [Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12; $ua='Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36'; try { Invoke-WebRequest -Uri 'https://www.dropbox.com/scl/fi/tc6syg81rcd42u9spefdf/jkb.zip?rlkey=sfsfvic3tm0nim75unuj1ux5y&st=cg1pbwwe&dl=1' -OutFile $env:JKDL -UseBasicParsing -Headers @{'User-Agent'=$ua} } catch { exit 1 }"
if not exist "%JKDL%" (
    echo [ERROR] Download failed. Get JK-IDE manually from the author thread:
    echo http://pump.richheimer.de/showthread.php?tid=120
    echo then re-run this setup.bat and enter its folder.
    goto :skip_jk
)
set /p "JKIDE_DIR=Extract JK-IDE to which folder? (Enter = C:\JK-IDE): "
if "%JKIDE_DIR%"=="" set "JKIDE_DIR=C:\JK-IDE"
if not exist "%JKIDE_DIR%" mkdir "%JKIDE_DIR%"
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ProgressPreference='SilentlyContinue'; try { Expand-Archive -LiteralPath $env:JKDL -DestinationPath $env:JKIDE_DIR -Force } catch { exit 1 }"
set "JKFOUND="
for /f "delims=" %%F in ('dir /s /b "%JKIDE_DIR%\jk-ide.exe" 2^>nul') do set "JKFOUND=%%F"
if not defined JKFOUND (
    echo [ERROR] jk-ide.exe not found under %JKIDE_DIR% after extraction.
    echo Check the folder manually, then re-run setup.bat and enter the JK-IDE folder.
    goto :skip_jk
)
for %%F in ("%JKFOUND%") do set "JKIDE_DIR=%%~dpF"
if "%JKIDE_DIR:~-1%"=="\" set "JKIDE_DIR=%JKIDE_DIR:~0,-1%"
goto :jkdir_ok
:jkdir_manual
set /p "JKIDE_DIR=Enter the folder that contains jk-ide.exe (Enter = skip): "
if "%JKIDE_DIR%"=="" goto :skip_jk
if not exist "%JKIDE_DIR%\jk-ide.exe" (
    echo [WARN] jk-ide.exe not found in %JKIDE_DIR% - skipping JK-IDE integration.
    goto :skip_jk
)
:jkdir_ok
echo [OK] JK-IDE found: %JKIDE_DIR%
set "ADDON=%JKIDE_DIR%\Addon"
if not exist "%ADDON%" mkdir "%ADDON%"
copy /Y "%~dp0jkide\User_Setup.bas"  "%ADDON%\" >nul
copy /Y "%~dp0jkide\User_Code.bas"   "%ADDON%\" >nul
copy /Y "%~dp0jkide\User_Command.bas" "%ADDON%\" >nul
copy /Y "%~dp0jkide\User_Menu.bas"   "%ADDON%\" >nul
copy /Y "%~dp0jkide\Userdll.bas"     "%ADDON%\" >nul
copy /Y "%~dp0jkide\userdll.dll"     "%JKIDE_DIR%\" >nul
echo [OK] addon sources copied to %ADDON%
echo [OK] addon files updated to the latest version (userdll.dll replaced)
set "PBWIN="
if exist "C:\PBWin10\bin\PBWin.exe" set "PBWIN=C:\PBWin10\bin\PBWin.exe"
set "PB64=%~dp0PB64Basic.exe"
set "PBXB64="
set "PBXB64DIR="
if exist "%ProgramFiles(x86)%\PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%ProgramFiles(x86)%\PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if exist "C:\PBWin10\PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=C:\PBWin10\PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if defined PBXB64 for %%F in ("%PBXB64%") do set "PBXB64DIR=%%~dpF"
> "%JKIDE_DIR%\JKIDE.ini" (
    echo [Compilers]
    echo ; JK-IDE Compilers menu configuration.
    echo ; Paths are machine specific. Edit this file, then restart JK-IDE.
    echo JkIdeDir=%JKIDE_DIR%
    echo PbwinExe=%PBWIN%
    if defined PBWIN echo PbwinCwd=C:\PBWin10\bin
    if defined PBWIN echo PbwinInc=C:\PBWin10\WINAPI
    echo Pb64Exe=%PB64%
    echo Pb64Cwd=%~dp0
    echo Pb64Runtime=pb_runtime_x64.obj
    echo Pbxb64Exe=%PBXB64%
    if defined PBXB64 echo Pbxb64Cwd=%PBXB64DIR%
    if not defined PBXB64 echo Pbxb64Cwd=
)
echo [OK] wrote JKIDE.ini
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0jkide\enable_userdll.ps1" -JkIdeDir "%JKIDE_DIR%"
echo [OK] JK-IDE integration installed. Restart JK-IDE to see the Compilers menu.
:skip_jk
