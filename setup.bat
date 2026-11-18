@echo off
setlocal
set "PB64_INCLUDE=%~1"
"%SystemRoot%\System32\chcp.com" 65001 >nul
title PB64Basic Setup
echo ==================================================
echo   PB64Basic - One-click Setup
echo ==================================================
echo.
echo This script checks and installs:
echo   1. Rust toolchain       (if missing)
echo   2. LLVM / Clang         (if missing, ~500 MB download)
echo   3. Windows SDK          (if missing)
echo   4. Checks for PowerBASIC WinAPI includes (optional)
echo   5. Builds pb_runtime_x64.obj / pb_runtime_x86.obj and PB64Basic.exe
echo.
echo Optional: pass a PowerBASIC install root or WinAPI folder as the first argument,
echo for example: setup.bat "C:\PBWin10" or setup.bat "C:\PBWin10\WINAPI"
echo PB64Basic does not copy or redistribute those headers.
echo.
echo NOTE: If a Windows UAC prompt appears, click YES to
echo allow the installers to run.
echo.
pause
if defined PB64_INCLUDE (
  "%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup.ps1" -IncludeDir "%PB64_INCLUDE%"
) else (
  "%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup.ps1"
)
echo.
echo ==================================================
echo   Optional: JK-IDE integration (Compilers menu)
echo ==================================================
echo This installs the PB64Basic Compilers menu into JK-IDE so you can
echo compile the active file with PBWin 10 / PB64Basic / PBXB64.
echo Type Y to install, anything else to skip.
set /p "JKDO=Install JK-IDE integration? (Y/N): "
if /I not "%JKDO%"=="Y" goto :skip_jk
rem Look for JK-IDE on every local drive. The old check only looked at
rem "%ProgramFiles(x86)%\JK-IDE" and "C:\PBWin10\JK-IDE", so a normal install
rem such as D:\JK-IDE was reported as "JK-IDE not found".
echo Searching for JK-IDE on all local drives...
set "JKIDE_DIR="
set "JKIDE_COUNT=0"
set "JK_TMP=%TEMP%\pb64_jkide_dir.txt"
if exist "%JK_TMP%" del "%JK_TMP%" >nul 2>nul
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0jkide\find_jkide.ps1" > "%JK_TMP%" 2>nul
if not exist "%JK_TMP%" goto :jk_ask_dir
for /f %%C in ('type "%JK_TMP%" ^| "%SystemRoot%\System32\find.exe" /c /v ""') do set "JKIDE_COUNT=%%C"
if "%JKIDE_COUNT%"=="0" goto :jk_not_found
if "%JKIDE_COUNT%"=="1" goto :jk_pick_one

rem Several installs: list them, best first, and let the user choose.
echo Multiple JK-IDE installations were found:
"%SystemRoot%\System32\findstr.exe" /n "^" "%JK_TMP%"
set "JK_PICK="
set /p "JK_PICK=Which one should be used? [1]: "
if not defined JK_PICK set "JK_PICK=1"
for /f "delims=0123456789" %%A in ("%JK_PICK%") do set "JK_PICK=1"
set /a JK_SKIP=JK_PICK-1
if %JK_SKIP% LSS 1 set "JK_SKIP=0"
if %JK_SKIP% GEQ %JKIDE_COUNT% set "JK_SKIP=0"
if "%JK_SKIP%"=="0" goto :jk_pick_first
set "JKIDE_DIR="
for /f "usebackq skip=%JK_SKIP% delims=" %%L in ("%JK_TMP%") do if not defined JKIDE_DIR set "JKIDE_DIR=%%L"
goto :jk_pick_after

:jk_pick_first
set /p "JKIDE_DIR=" < "%JK_TMP%"

:jk_pick_after
if not defined JKIDE_DIR goto :jk_not_found
if not exist "%JKIDE_DIR%\jk-ide.exe" goto :jk_not_found
del "%JK_TMP%" >nul 2>nul
if not exist "%LOCALAPPDATA%\PB64Basic" md "%LOCALAPPDATA%\PB64Basic" >nul 2>nul
> "%LOCALAPPDATA%\PB64Basic\jkide_dir.txt" echo %JKIDE_DIR%
goto :jkdir_ok

:jk_pick_one
set /p "JKIDE_DIR=" < "%JK_TMP%"
del "%JK_TMP%" >nul 2>nul
if not defined JKIDE_DIR goto :jk_ask_dir
if not exist "%JKIDE_DIR%\jk-ide.exe" goto :jk_ask_dir
goto :jkdir_ok

:jk_not_found
del "%JK_TMP%" >nul 2>nul

:jk_ask_dir
echo JK-IDE was not found in the usual folders or on any local drive.
set /p "JKIDE_DIR=Enter the folder that contains jk-ide.exe (Enter = download / skip): "
if not defined JKIDE_DIR goto :jk_dl
if exist "%JKIDE_DIR%\jk-ide.exe" goto :jkdir_ok
echo [WARN] jk-ide.exe not found in "%JKIDE_DIR%".
set "JKIDE_DIR="
goto :jk_dl
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
rem Paths that worked before must survive a re-run even when detection fails.
rem The old setup.bat rewrote JKIDE.ini from scratch, so answering "skip" to
rem an undetected compiler blanked Pbxb64Exe/Pbxb64Cwd and broke the
rem Compilers > PBXB64 menu. Read the previous values first and reuse them.
set "OLD_PBWIN="
set "OLD_PB64="
set "OLD_PBX="
if not exist "%JKIDE_DIR%\JKIDE.ini" goto :old_ini_done
"%SystemRoot%\System32\findstr.exe" /b /i "PbwinExe= Pb64Exe= Pbxb64Exe=" "%JKIDE_DIR%\JKIDE.ini" > "%TEMP%\pb64_old_ini.txt" 2>nul
if not exist "%TEMP%\pb64_old_ini.txt" goto :old_ini_done
for /f "usebackq tokens=1,* delims==" %%A in ("%TEMP%\pb64_old_ini.txt") do (
    if /i "%%A"=="PbwinExe" set "OLD_PBWIN=%%B"
    if /i "%%A"=="Pb64Exe" set "OLD_PB64=%%B"
    if /i "%%A"=="Pbxb64Exe" set "OLD_PBX=%%B"
)
del "%TEMP%\pb64_old_ini.txt" >nul 2>nul
:old_ini_done
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
if defined OLD_PBWIN if exist "%OLD_PBWIN%" set "PBWIN=%OLD_PBWIN%"
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
if not defined PB64 if defined OLD_PB64 if exist "%OLD_PB64%" set "PB64=%OLD_PB64%"
set "PBXB64="
set "PBXB64DIR="
set "PBX_INPUT="
if exist "%ProgramFiles(x86)%\PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%ProgramFiles(x86)%\PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if exist "C:\PBWin10\PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=C:\PBWin10\PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 for /d %%V in ("%~dp0PBXB64\V*") do if not defined PBXB64 if exist "%%~fV\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%%~fV\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 for /d %%V in ("%~dp0..\..\PBXB64\V*") do if not defined PBXB64 if exist "%%~fV\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%%~fV\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 if exist "%~dp0PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%~dp0PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 if exist "%~dp0..\..\PBXB64\PBX-Editor\PBXB64\PBXB64.exe" set "PBXB64=%~dp0..\..\PBXB64\PBX-Editor\PBXB64\PBXB64.exe"
if not defined PBXB64 if defined OLD_PBX if exist "%OLD_PBX%" set "PBXB64=%OLD_PBX%"
if defined PBXB64 goto :pbx_set
rem Personal installs often live under C:\Users\<name> (this machine:
rem C:\Users\HANLO\Doubao\...\PBXB64\V6\PBX-Editor\PBXB64).  The fixed paths
rem above cannot see those, so ask the locator to search the user profile.
set "PBX_TMP=%TEMP%\pb64_pbxb64_exe.txt"
if exist "%PBX_TMP%" del "%PBX_TMP%" >nul 2>nul
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0jkide\find_pbxb64.ps1" -RepoRoot "%~dp0" > "%PBX_TMP%" 2>nul
if exist "%PBX_TMP%" set /p "PBXB64=" < "%PBX_TMP%"
if exist "%PBX_TMP%" del "%PBX_TMP%" >nul 2>nul
if defined PBXB64 goto :pbx_set
echo.
echo PBXB64 was not found in the standard locations.
set /p "PBX_INPUT=Enter the full path to PBXB64.exe, or press Enter to skip: "
if not defined PBX_INPUT goto :pbx_set
if exist "%PBX_INPUT%\PBXB64.exe" set "PBX_INPUT=%PBX_INPUT%\PBXB64.exe"
if exist "%PBX_INPUT%" set "PBXB64=%PBX_INPUT%"
if defined PBXB64 goto :pbx_set
echo [WARN] The file "%PBX_INPUT%" does not exist - skipping PBXB64.
:pbx_set
if defined PBXB64 for %%F in ("%PBXB64%") do set "PBXB64DIR=%%~dpF"
set "PB64DIR="
if defined PB64 for %%F in ("%PB64%") do set "PB64DIR=%%~dpF"
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
if exist "%~dp0jkide\PB64Basic.chm" (
  echo [OK] bundled PB64Basic help found - install_help.ps1 will deploy it
) else if defined PBWIN_BIN (
  if exist "%PBWIN_BIN%PBWin.chm" copy /Y "%PBWIN_BIN%PBWin.chm" "%JKIDE_DIR%\PBWin.chm" >nul
  if exist "%PBWIN_BIN%PBWin.chw" copy /Y "%PBWIN_BIN%PBWin.chw" "%JKIDE_DIR%\PBWin.chw" >nul
  if exist "%PBWIN_BIN%PBWin.chm" copy /Y "%PBWIN_BIN%PBWin.chm" "%JKIDE_DIR%\PB.chm" >nul
  if exist "%PBWIN_BIN%PBWin.chw" copy /Y "%PBWIN_BIN%PBWin.chw" "%JKIDE_DIR%\PB.chw" >nul
  echo [OK] PBWin.chm/.chw and PB.chm/.chw copied to %JKIDE_DIR%
)
if not exist "%~dp0jkide\PB64Basic.chm" if not defined PBWIN_BIN echo [WARN] PBWin not found - skipped copying help files
> "%JKIDE_DIR%\JKIDE.ini" (
    echo [Compilers]
    echo ; JK-IDE Compilers menu configuration.
    echo ; Paths are machine specific. Edit this file, then restart JK-IDE.
    echo JkIdeDir=%JKIDE_DIR%
    echo PbwinExe=%PBWIN%
    if defined PBWIN echo PbwinCwd=%PBWIN_CWD%
    if defined PBWIN echo PbwinInc=%PBWIN_INC%
    echo Pb64Exe=%PB64%
    echo Pb64Cwd=%PB64DIR%
    echo Pb64Runtime=pb_runtime_x64.obj
    echo Pb64RuntimeX86=pb_runtime_x86.obj
    echo Pbxb64Exe=%PBXB64%
    if defined PBXB64 echo Pbxb64Cwd=%PBXB64DIR%
    if not defined PBXB64 echo Pbxb64Cwd=
    rem PBWin.chm is the native PBWin 10 help and the safe target. install_help
    rem rewrites PbHelpFile only after a successful copy; if a locked file blocks
    rem the copy, the previous usable value stays instead of pointing at a file
    rem that was never created.
    if exist "%JKIDE_DIR%\PBWin.chm" (
        echo PbHelpFile=%JKIDE_DIR%\PBWin.chm
    ) else (
        echo PbHelpFile=%JKIDE_DIR%\PB.chm
    )
)
echo [OK] wrote JKIDE.ini
if exist "%~dp0jkide\PB64Basic.chm" (
    "%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0jkide\install_help.ps1" -JkIdeDir "%JKIDE_DIR%" -SourceDir "%~dp0jkide"
    if errorlevel 1 echo [WARN] PB64Basic help installation failed; PbHelpFile remains the native PBWin fallback
)
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0jkide\enable_userdll.ps1" -JkIdeDir "%JKIDE_DIR%"
echo [OK] JK-IDE integration installed. Restart JK-IDE to see the Compilers menu.
:skip_jk
