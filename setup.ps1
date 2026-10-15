# PB64Basic - One-click dependency setup and build
#
# Checks / installs (via winget when missing):
#   1. Rust toolchain        (Rustlang.Rustup)
#   2. LLVM / Clang          (LLVM.LLVM, ~500 MB)
#   3. Windows SDK           (Microsoft.WindowsSDK.10.0.26100)
# Then builds:
#   4. pb_runtime_x64.obj    (from pbcompiler/runtime/pb_runtime.c)
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
$clang = Get-Command clang -ErrorAction SilentlyContinue
if (-not $clang) {
    $llvmBin = "C:\Program Files\LLVM\bin\clang.exe"
    if (Test-Path $llvmBin) { $clang = Get-Command $llvmBin -ErrorAction SilentlyContinue }
}
if ($clang) {
    $ver = & $clang.Source --version 2>$null | Select-Object -First 1
    Write-Ok "Clang found: $ver"
} else {
    Write-Warn "LLVM not found. Installing via winget (LLVM.LLVM, ~500 MB download)..."
    Write-Warn "If a UAC prompt appears, click YES."
    winget install --id LLVM.LLVM -e --accept-source-agreements --accept-package-agreements --silent 2>&1 | Out-Null
    $llvmBin = "C:\Program Files\LLVM\bin\clang.exe"
    if (Test-Path $llvmBin) { Write-Ok "LLVM installed."; $changed = $true }
    else { Write-Fail "LLVM install failed. Install LLVM manually, then re-run this script." }
}
# Make sure clang is on PATH for this session after a fresh install
$env:Path = "C:\Program Files\LLVM\bin;" + $env:Path

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

# ---------- 4. Build runtime object ----------
Write-Step "Building pb_runtime_x64.obj..."
$rtSrc = Join-Path $ScriptDir "pbcompiler\runtime\pb_runtime.c"
$rtObj = Join-Path $ScriptDir "pb_runtime_x64.obj"
if (Test-Path $rtSrc) {
    & clang -c $rtSrc -o $rtObj --target=x86_64-pc-windows-msvc 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0 -and (Test-Path $rtObj)) {
        Write-Ok "Runtime object built: $rtObj"
    } else {
        Write-Fail "Runtime build failed. LLVM/Clang is required (step 2)."
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
