# PB64Basic - One-click dependency setup and build
#
# Checks / installs (via winget when missing):
#   1. Rust toolchain        (Rustlang.Rustup)
#   2. LLVM / Clang          (LLVM.LLVM, ~500 MB)
#   3. Windows SDK           (Microsoft.WindowsSDK.10.0.26100)
# Then builds:
#   4. pb_runtime_x64.obj / pb_runtime_x86.obj (from pbcompiler/runtime/pb_runtime.c)
#   5. PB64Basic.exe        (release build via cargo)
#
# Run from setup.bat (double-click), or:
#   powershell -NoProfile -ExecutionPolicy Bypass -File setup.ps1

$ErrorActionPreference = 'Continue'
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

function Write-Step($msg)  { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)    { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Warn($msg)  { Write-Host "   [!] $msg" -ForegroundColor Yellow }
function Write-Fail($msg)  { Write-Host "   [X] $msg" -ForegroundColor Red }

$changed = $false

# ---------- 0. Check winget (App Installer) ----------
Write-Step "Checking winget (used to auto-install dependencies)..."
$winget = Get-Command winget -ErrorAction SilentlyContinue
if ($winget) {
    Write-Ok "winget is available - Rust / LLVM / Windows SDK can be installed automatically."
} else {
    Write-Warn "winget not found: the 3 dependencies below cannot be auto-installed if missing and must be handled manually:"
    Write-Warn "   Rust:        https://rustup.rs"
    Write-Warn "   LLVM:        https://releases.llvm.org"
    Write-Warn "   Windows SDK: https://developer.microsoft.com/windows/downloads/windows-sdk/"
    Write-Warn "(Windows 10 2004+/Windows 11 ship with winget; on older systems install App Installer from the Microsoft Store, then re-run this script)"
}

# ---------- 1. Rust toolchain ----------
Write-Step "Checking Rust toolchain..."
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path $cargoPath) { $cargo = Get-Command $cargoPath -ErrorAction SilentlyContinue }
}
if ($cargo) {
    $ver = & $cargo.Source --version 2>$null
    Write-Ok "Rust found: $ver"
} else {
    Write-Warn "Rust not found. Installing via winget (Rustlang.Rustup)..."
    winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements --silent 2>&1 | Out-Null
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path $cargoPath) { Write-Ok "Rust installed."; $changed = $true }
    else { Write-Fail "Rust install failed. Re-run this script after installing rustup from https://rustup.rs" }
}

# ---------- 2. LLVM / Clang ----------
Write-Step "Checking LLVM / Clang..."
$llvmCandidates = @(
    "C:\Program Files\LLVM\bin",
    "C:\Program Files (x86)\LLVM\bin",
    (Join-Path $env:LOCALAPPDATA "LLVM\bin"),
    (Join-Path $env:USERPROFILE "AppData\Local\LLVM\bin")
)
$clangExe = $null
$clang = Get-Command clang -ErrorAction SilentlyContinue
if ($clang) {
    $clangExe = $clang.Source
} else {
    foreach ($bin in $llvmCandidates) {
        $probe = Join-Path $bin "clang.exe"
        if (Test-Path $probe) { $clangExe = $probe; break }
    }
}
if ($clangExe) {
    $ver = & $clangExe --version 2>$null | Select-Object -First 1
    Write-Ok "Clang found: $ver"
    # Make sure clang's bin folder is on PATH for this session (fresh install / non-standard layout)
    $clangBin = Split-Path -Parent $clangExe
    if (-not (($env:Path -split ';') -contains $clangBin)) {
        $env:Path = "$clangBin;" + $env:Path
    }
} else {
    Write-Warn "LLVM not found. Installing via winget (LLVM.LLVM, ~500 MB download)..."
    Write-Warn "If a UAC prompt appears, click YES."
    winget install --id LLVM.LLVM -e --accept-source-agreements --accept-package-agreements --silent 2>&1 | Out-Null
    foreach ($bin in $llvmCandidates) {
        $probe = Join-Path $bin "clang.exe"
        if (Test-Path $probe) { $clangExe = $probe; break }
    }
    if ($clangExe) {
        Write-Ok "LLVM installed."; $changed = $true
        $clangBin = Split-Path -Parent $clangExe
        $env:Path = "$clangBin;" + $env:Path
    } else {
        Write-Fail "LLVM install failed. Install LLVM manually, then re-run this script."
    }
}

# ---------- 3. Windows SDK ----------
Write-Step "Checking Windows SDK..."
$sdkRoots = @(
    "C:\Program Files (x86)\Windows Kits\10\Lib",
    "C:\Program Files\Windows Kits\10\Lib"
)
$sdkFound = $false
foreach ($r in $sdkRoots) {
    if (Test-Path $r) {
        foreach ($v in (Get-ChildItem $r -Directory -ErrorAction SilentlyContinue)) {
            if (Test-Path (Join-Path $v.FullName "um\x64")) {
                Write-Ok "Windows SDK found: $($v.Name) (um\x64 present)"
                $sdkFound = $true; break
            }
        }
    }
    if ($sdkFound) { break }
}
if (-not $sdkFound) {
    Write-Warn "Windows SDK not found. Installing Windows 11 SDK (10.0.26100) via winget..."
    Write-Warn "If a UAC prompt appears, click YES."
    winget install --id Microsoft.WindowsSDK.10.0.26100 -e --accept-source-agreements --accept-package-agreements --silent 2>&1 | Out-Null
    foreach ($r in $sdkRoots) {
        if (Test-Path $r) {
            foreach ($v in (Get-ChildItem $r -Directory -ErrorAction SilentlyContinue)) {
                if (Test-Path (Join-Path $v.FullName "um\x64")) { Write-Ok "Windows SDK installed: $($v.Name)"; $sdkFound = $true; break }
            }
        }
        if ($sdkFound) { break }
    }
    if (-not $sdkFound) { Write-Fail "SDK install failed. Install the Windows SDK manually from https://developer.microsoft.com/windows/downloads/windows-sdk/" }
}

# ---------- 4. Build runtime objects ----------
$rtSrc = Join-Path $ScriptDir "pbcompiler\runtime\pb_runtime.c"
if (Test-Path $rtSrc) {
    if (-not $clangExe -or -not (Test-Path $clangExe)) {
        Write-Fail "clang is not available (step 2): install LLVM/Clang and add its bin folder to PATH, then re-run."
    } else {
        # -ffunction-sections/-fdata-sections (batch 310): place each function and
        # global in its own COMDAT section so the linker (/OPT:REF) drops the runtime
        # functions the program never references, shrinking the EXE instead of
        # pulling the whole runtime. -O2 shrinks the object ~20% over -O0.
        foreach ($rtTarget in @(
            @{ Obj = "pb_runtime_x64.obj"; Triple = "x86_64-pc-windows-msvc" },
            @{ Obj = "pb_runtime_x86.obj"; Triple = "i686-pc-windows-msvc" }
        )) {
            $objName = $rtTarget.Obj
            $triple  = $rtTarget.Triple
            $rtObj = Join-Path $ScriptDir $objName
            Write-Step "Building $objName..."
            $rtOut = & $clangExe -c $rtSrc -o $rtObj --target=$triple -O2 -ffunction-sections -fdata-sections 2>&1
            $rtCode = $LASTEXITCODE
            if ($rtCode -eq 0 -and (Test-Path $rtObj)) {
                Write-Ok "Runtime object built: $rtObj"
            } else {
                Write-Fail "Runtime build failed ($triple): clang exited with code $rtCode."
                $errLines = @($rtOut | Where-Object { $_ -match "error:|fatal error|No such file|not found" })
                if ($errLines.Count -gt 0) {
                    Write-Host "   clang diagnostics:" -ForegroundColor Red
                    foreach ($e in ($errLines | Select-Object -First 12)) { Write-Host "     $e" -ForegroundColor Red }
                }
                Write-Warn "   If clang could not start: LLVM/Clang is missing or not on PATH (step 2)."
                Write-Warn "   If clang started but reported missing headers: install the Windows SDK and MSVC build tools."
                Write-Warn "   If the error names an unknown target or invalid argument: this is a setup-script invocation problem, not a missing dependency."
            }
        }
    }
} else {
    Write-Fail "pb_runtime.c not found: $rtSrc"
}

# ---------- 5. Build the compiler ----------
Write-Step "Building pbcompiler (release)..."
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path $cargoPath) { $env:Path = (Join-Path $env:USERPROFILE ".cargo\bin") + ";" + $env:Path }
}
cargo build --release -p pbcompiler
if ($LASTEXITCODE -eq 0) { Write-Ok "pbcompiler built." }
else { Write-Fail "PB64Basic build failed. Rust toolchain is required (step 1)." }

# ---------- 6. Summary ----------
Write-Step "Setup summary"
$pb = Join-Path $ScriptDir "target\release\PB64Basic.exe"
if (Test-Path $pb) {
    Write-Ok "Compiler ready: $pb"
    Write-Host ""
    Write-Host "Try it (from this folder):" -ForegroundColor White
    Write-Host "   PB64Basic build your_program.bas --exe --target x86_64-pc-windows-msvc --runtime-lib pb_runtime_x64.obj" -ForegroundColor Gray
    Write-Host "   (--lib-dir is optional: the compiler auto-detects the Windows SDK)" -ForegroundColor Gray
} else {
    Write-Fail "PB64Basic.exe not found after the build. Check the errors above."
}
Write-Host ""
Write-Host "Setup finished." -ForegroundColor Cyan
