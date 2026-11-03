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
if defined PBWIN goto :pbwin_set
echo.
echo PBWin was not found at the default path C:\PBWin10\bin\PBWin.exe
echo It may be installed in another folder.
set /p "PBWIN_INPUT=Enter the full path to PBWin.exe, or press Enter to skip: "
if not defined PBWIN_INPUT goto :pbwin_skip
if exist "%PBWIN_INPUT%" set "PBWIN=%PBWIN_INPUT%"
if defined PBWIN goto :pbwin_set
echo [WARN] The file "%PBWIN_INPUT%" does not exist - skipping PBWin.
:pbwin_skip
echo [WARN] PBWin not configured - PB.chm/PB.chw will not be copied.
:pbwin_set
set "PB64="
if exist "%~dp0PB64Basic.exe" set "PB64=%~dp0PB64Basic.exe"
if not defined PB64 if exist "%~dp0target\release\PB64Basic.exe" set "PB64=%~dp0target\release\PB64Basic.exe"
set "PBXB64="
set "PBXB64DIR="
set "PBX_INPUT="
if exist "%ProgramFiles(x86)%\PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%ProgramFiles(x86)%\PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if exist "C:\PBWin10\PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=C:\PBWin10\PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 for /d %%V in ("%~dp0PBXB64\V*") do if not defined PBXB64 if exist "%%~fV\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%%~fV\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 for /d %%V in ("%~dp0..\..\PBXB64\V*") do if not defined PBXB64 if exist "%%~fV\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%%~fV\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 if exist "%~dp0PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%~dp0PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 if exist "%~dp0..\..\PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%~dp0..\..\PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if defined PBXB64 goto :pbx_set
echo.
echo PBXB64 was not found in the standard locations.
set /p "PBX_INPUT=Enter the full path to PBXB64.exe, or press Enter to skip: "
if not defined PBX_INPUT goto :pbx_set
if exist "%PBX_INPUT%" set "PBXB64=%PBX_INPUT%"
if defined PBXB64 goto :pbx_set
echo [WARN] The file "%PBX_INPUT%" does not exist - skipping PBXB64.
:pbx_set
if defined PBXB64 for %%F in ("%PBXB64%") do set "PBXB64DIR=%%~dpF"
set "PBWIN_BIN="
if defined PBWIN for %%F in ("%PBWIN%") do set "PBWIN_BIN=%%~dpF"
set "PBWIN_CWD="
set "PBWIN_ROOT="
set "PBWIN_INC="
if not defined PBWIN_BIN goto :pbwin_paths_done
set "PBWIN_CWD=%PBWIN_BIN:~0,-1%"
for %%F in ("%PBWIN_CWD%") do set "PBWIN_ROOT=%%~dpF"
set "PBWIN_INC=%PBWIN_ROOT%WINAPI"
:pbwin_paths_done
if defined PBWIN_BIN (
  if exist "%PBWIN_BIN%PBWin.chm" copy /Y "%PBWIN_BIN%PBWin.chm" "%JKIDE_DIR%\PB.chm" >nul
  if exist "%PBWIN_BIN%PBWin.chw" copy /Y "%PBWIN_BIN%PBWin.chw" "%JKIDE_DIR%\PB.chw" >nul
  echo [OK] PB.chm and PB.chw copied to %JKIDE_DIR%
)
if not defined PBWIN_BIN echo [WARN] PBWin not found - skipped copying PB.chm/PB.chw
> "%JKIDE_DIR%\JKIDE.ini" (
    echo [Compilers]
    echo ; JK-IDE Compilers menu configuration.
    echo ; Paths are machine specific. Edit this file, then restart JK-IDE.
    echo JkIdeDir=%JKIDE_DIR%
    echo PbwinExe=%PBWIN%
    if defined PBWIN echo PbwinCwd=%PBWIN_CWD%
    if defined PBWIN echo PbwinInc=%PBWIN_INC%
    echo Pb64Exe=%PB64%
    echo Pb64Cwd=%~dp0
    echo Pb64Runtime=pb_runtime_x64.obj
    echo Pbxb64Exe=%PBXB64%
    if defined PBXB64 echo Pbxb64Cwd=%PBXB64DIR%
    if not defined PBXB64 echo Pbxb64Cwd=
    echo PbHelpFile=%JKIDE_DIR%\PB.chm
)
echo [OK] wrote JKIDE.ini
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0jkide\enable_userdll.ps1" -JkIdeDir "%JKIDE_DIR%"
echo [OK] JK-IDE integration installed. Restart JK-IDE to see the Compilers menu.
:skip_jk
