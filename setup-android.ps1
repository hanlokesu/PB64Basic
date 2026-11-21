# PB64Basic - Android build toolchain installer
#
# Installs the four top-level components used by tools/build_apk.ps1:
#   1. Eclipse Temurin JDK 17.0.21+9
#   2. Gradle 8.14.3
#   3. Android SDK command-line tools 15859902, then
#      platform-tools, platforms android-34/35 and build-tools 34.0.0/35.0.0
#   4. Android NDK r27c (27.2.12479018)
#
# Download size is about 1.3 GB; the installed tree is about 3 GB.
#
# Usage:
#   powershell -NoProfile -ExecutionPolicy Bypass -File setup-android.ps1
#   powershell -NoProfile -ExecutionPolicy Bypass -File setup-android.ps1 -CheckOnly
#   powershell -NoProfile -ExecutionPolicy Bypass -File setup-android.ps1 -Root D:\AndroidToolchain
#   powershell -NoProfile -ExecutionPolicy Bypass -File setup-android.ps1 -NoEnv
#
# -CheckOnly reports what is present and what would be installed; it changes
# nothing. -NoEnv installs the files but leaves JAVA_HOME / ANDROID_HOME /
# ANDROID_NDK_HOME / GRADLE_HOME and the user PATH untouched.

param(
    [string]$Root = "",
    [switch]$CheckOnly,
    [switch]$NoEnv,
    [switch]$Force,
    [switch]$SkipSdkPackages,
    [switch]$SkipNdk
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$JdkVersion = '17.0.21+9'
$GradleVersion = '8.14.3'
$NdkVersion = '27.2.12479018'
$NdkRelease = 'r27c'

function Write-Step($msg)  { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Write-Ok($msg)    { Write-Host "   [OK] $msg" -ForegroundColor Green }
function Write-Warn($msg)  { Write-Host "   [!] $msg" -ForegroundColor Yellow }
function Write-Fail($msg)  { Write-Host "   [X] $msg" -ForegroundColor Red }

function Resolve-InstallRoot {
    param([string]$Explicit)

    if (-not [string]::IsNullOrWhiteSpace($Explicit)) {
        return [IO.Path]::GetFullPath($Explicit)
    }
    if (-not [string]::IsNullOrWhiteSpace($env:PB64_ANDROID_HOME)) {
        return [IO.Path]::GetFullPath($env:PB64_ANDROID_HOME)
    }
    if (Test-Path -LiteralPath 'D:\') {
        return 'D:\AndroidToolchain'
    }
    return (Join-Path $env:LOCALAPPDATA 'PB64Basic\AndroidToolchain')
}

$RootExplicit = -not [string]::IsNullOrWhiteSpace($Root)
$InstallRoot = Resolve-InstallRoot $Root
$DownloadDir = Join-Path $InstallRoot '_downloads'
$StagingDir = Join-Path $DownloadDir 'staging'

function Test-JdkHome {
    param([string]$Candidate)
    if ([string]::IsNullOrWhiteSpace($Candidate)) { return $false }
    if (-not (Test-Path -LiteralPath (Join-Path $Candidate 'bin\java.exe'))) { return $false }
    $release = Join-Path $Candidate 'release'
    if (-not (Test-Path -LiteralPath $release)) { return $false }
    $text = Get-Content -LiteralPath $release -Raw
    return ($text -match 'IMPLEMENTOR_VERSION="Temurin-17\.0\.21\+9"')
}

function Test-GradleHome {
    param([string]$Candidate)
    if ([string]::IsNullOrWhiteSpace($Candidate)) { return $false }
    if (-not (Test-Path -LiteralPath (Join-Path $Candidate 'bin\gradle.bat'))) { return $false }
    return (Test-Path -LiteralPath (Join-Path $Candidate "lib\gradle-launcher-$GradleVersion.jar"))
}

function Test-NdkRoot {
    param([string]$NdkRoot)
    if ([string]::IsNullOrWhiteSpace($NdkRoot)) { return $false }
    $clang = Join-Path $NdkRoot 'toolchains\llvm\prebuilt\windows-x86_64\bin\clang.exe'
    if (-not (Test-Path -LiteralPath $clang)) { return $false }
    $props = Join-Path $NdkRoot 'source.properties'
    if (-not (Test-Path -LiteralPath $props)) { return $false }
    return ((Get-Content -LiteralPath $props -Raw) -match "Pkg\.Revision = $([regex]::Escape($NdkVersion))")
}

function Test-SdkComplete {
    param([string]$SdkRoot)
    if ([string]::IsNullOrWhiteSpace($SdkRoot)) { return $false }
    $required = @(
        'platform-tools\adb.exe',
        'platforms\android-34\android.jar',
        'platforms\android-35\android.jar',
        'build-tools\34.0.0\aapt2.exe',
        'build-tools\35.0.0\aapt2.exe',
        'build-tools\35.0.0\zipalign.exe',
        'build-tools\35.0.0\apksigner.bat'
    )
    foreach ($rel in $required) {
        if (-not (Test-Path -LiteralPath (Join-Path $SdkRoot $rel))) { return $false }
    }
    return $true
}

function Find-JdkHome {
    param([string[]]$Roots)
    foreach ($r in $Roots) {
        if ([string]::IsNullOrWhiteSpace($r)) { continue }
        $base = Join-Path $r 'jdk'
        if (-not (Test-Path -LiteralPath $base)) { continue }
        $dirs = Get-ChildItem -LiteralPath $base -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending
        foreach ($d in $dirs) {
            if (Test-JdkHome $d.FullName) { return $d.FullName }
        }
    }
    if (-not $RootExplicit -and (Test-JdkHome $env:JAVA_HOME)) { return $env:JAVA_HOME }
    return ''
}

function Find-GradleHome {
    param([string[]]$Roots)
    foreach ($r in $Roots) {
        if ([string]::IsNullOrWhiteSpace($r)) { continue }
        $base = Join-Path $r 'gradle'
        if (-not (Test-Path -LiteralPath $base)) { continue }
        $dirs = Get-ChildItem -LiteralPath $base -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending
        foreach ($d in $dirs) {
            if (Test-GradleHome $d.FullName) { return $d.FullName }
        }
    }
    if (-not $RootExplicit -and (Test-GradleHome $env:GRADLE_HOME)) { return $env:GRADLE_HOME }
    return ''
}

function Find-SdkRoot {
    param([string[]]$Roots)
    foreach ($r in $Roots) {
        if ([string]::IsNullOrWhiteSpace($r)) { continue }
        $sdk = Join-Path $r 'Sdk'
        if (Test-Path -LiteralPath $sdk) { return $sdk }
    }
    if (-not $RootExplicit) {
        foreach ($envSdk in @($env:ANDROID_HOME, $env:ANDROID_SDK_ROOT)) {
            if (-not [string]::IsNullOrWhiteSpace($envSdk) -and (Test-Path -LiteralPath $envSdk)) {
                return [IO.Path]::GetFullPath($envSdk)
            }
        }
    }
    return ''
}

function Find-NdkRoot {
    param([string[]]$Roots, [string]$SdkRoot)
    foreach ($r in $Roots) {
        if ([string]::IsNullOrWhiteSpace($r)) { continue }
        $base = Join-Path $r 'ndk'
        if (-not (Test-Path -LiteralPath $base)) { continue }
        $dirs = Get-ChildItem -LiteralPath $base -Directory -ErrorAction SilentlyContinue |
            Sort-Object Name -Descending
        foreach ($d in $dirs) {
            if (Test-NdkRoot $d.FullName) { return $d.FullName }
        }
    }
    if (-not [string]::IsNullOrWhiteSpace($SdkRoot)) {
        $base = Join-Path $SdkRoot 'ndk'
        if (Test-Path -LiteralPath $base) {
            $dirs = Get-ChildItem -LiteralPath $base -Directory -ErrorAction SilentlyContinue |
                Sort-Object Name -Descending
            foreach ($d in $dirs) {
                if (Test-NdkRoot $d.FullName) { return $d.FullName }
            }
        }
    }
    if (-not $RootExplicit -and (Test-NdkRoot $env:ANDROID_NDK_HOME)) { return $env:ANDROID_NDK_HOME }
    return ''
}

function Find-SdkManager {
    param([string]$SdkRoot)
    if ([string]::IsNullOrWhiteSpace($SdkRoot)) { return '' }
    $latest = Join-Path $SdkRoot 'cmdline-tools\latest\bin\sdkmanager.bat'
    if (Test-Path -LiteralPath $latest) { return $latest }
    $base = Join-Path $SdkRoot 'cmdline-tools'
    if (-not (Test-Path -LiteralPath $base)) { return '' }
    foreach ($d in (Get-ChildItem -LiteralPath $base -Directory -ErrorAction SilentlyContinue)) {
        $candidate = Join-Path $d.FullName 'bin\sdkmanager.bat'
        if (Test-Path -LiteralPath $candidate) { return $candidate }
    }
    return ''
}

function Get-CheckedDownload {
    param(
        [string]$Name,
        [string]$Url,
        [long]$Size,
        [string]$Hash,
        [string]$Algorithm
    )

    if (-not (Test-Path -LiteralPath $DownloadDir)) {
        New-Item -ItemType Directory -Path $DownloadDir -Force | Out-Null
    }
    $path = Join-Path $DownloadDir $Name
    $valid = $false
    if (Test-Path -LiteralPath $path) {
        $item = Get-Item -LiteralPath $path
        if ($item.Length -eq $Size) {
            $actual = (Get-FileHash -LiteralPath $path -Algorithm $Algorithm).Hash
            if ($actual -ieq $Hash) { $valid = $true }
        }
    }
    if ($valid) {
        Write-Ok "Using cached $Name"
        return $path
    }
    if (Test-Path -LiteralPath $path) {
        Remove-Item -LiteralPath $path -Force
    }

    Write-Host ("   downloading {0} ({1:N0} MB)..." -f $Name, ($Size / 1MB)) -ForegroundColor Gray
    $headers = @{ 'User-Agent' = 'PB64Basic-setup/1.0' }
    $lastError = $null
    for ($attempt = 1; $attempt -le 2; $attempt++) {
        try {
            Invoke-WebRequest -Uri $Url -OutFile $path -UseBasicParsing -Headers $headers -TimeoutSec 1800
            $lastError = $null
            break
        } catch {
            $lastError = $_
            if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
            if ($attempt -lt 2) { Start-Sleep -Seconds 5 }
        }
    }
    if ($null -ne $lastError) {
        throw "Download failed for $Name`: $($lastError.Exception.Message)"
    }

    $item = Get-Item -LiteralPath $path
    if ($item.Length -ne $Size) {
        Remove-Item -LiteralPath $path -Force
        throw "$Name has the wrong size: expected $Size bytes, got $($item.Length)"
    }
    $actual = (Get-FileHash -LiteralPath $path -Algorithm $Algorithm).Hash
    if ($actual -ine $Hash) {
        Remove-Item -LiteralPath $path -Force
        throw "$Name checksum mismatch: expected $Hash, got $actual"
    }
    Write-Ok "Verified $Name ($Algorithm)"
    return $path
}

function Expand-ArchiveToStaging {
    param([string]$ZipPath, [string]$Label)
    if (-not (Test-Path -LiteralPath $StagingDir)) {
        New-Item -ItemType Directory -Path $StagingDir -Force | Out-Null
    }
    $dest = Join-Path $StagingDir ("{0}-{1}" -f $Label, [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $dest -Force | Out-Null

    # Windows' bundled tar (libarchive) handles ZIP entries longer than MAX_PATH.
    # .NET Framework's ZipFile.ExtractToDirectory fails on the NDK's deeply
    # nested sources; LongPathsEnabled is 0 on a default Windows install.
    $tar = Join-Path $env:SystemRoot 'System32\tar.exe'
    if (Test-Path -LiteralPath $tar) {
        & $tar -xf $ZipPath -C $dest
        if ($LASTEXITCODE -ne 0) {
            throw "$Label archive extraction failed with tar (exit $LASTEXITCODE)"
        }
    } else {
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        [System.IO.Compression.ZipFile]::ExtractToDirectory($ZipPath, $dest)
    }

    $top = @(Get-ChildItem -LiteralPath $dest -Directory -ErrorAction SilentlyContinue)
    if ($top.Count -ne 1) {
        throw "$Label archive should contain one top-level folder; found $($top.Count)"
    }
    return $top[0].FullName
}

function Publish-Directory {
    param([string]$Source, [string]$Destination)

    if (Test-Path -LiteralPath $Destination) {
        if (-not $Force) {
            throw "$Destination already exists but is incomplete. Re-run with -Force to replace it."
        }
        $backup = "$Destination.old-$(Get-Date -Format 'yyyyMMddHHmmss')"
        Move-Item -LiteralPath $Destination -Destination $backup
        Write-Warn "Moved incomplete folder to $backup"
    }
    $parent = Split-Path -Parent $Destination
    if (-not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    Move-Item -LiteralPath $Source -Destination $Destination
    return $Destination
}

function Install-Jdk {
    $zip = Get-CheckedDownload `
        'OpenJDK17U-jdk_x64_windows_hotspot_17.0.21_9.zip' `
        'https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.21%2B9/OpenJDK17U-jdk_x64_windows_hotspot_17.0.21_9.zip' `
        191117380 `
        '98518ECF554465057EE5509462631DEEEE374BC368C12A411D225CD29381F84B' `
        'SHA256'
    $top = Expand-ArchiveToStaging $zip 'jdk'
    $name = Split-Path -Leaf $top
    $dest = Publish-Directory $top (Join-Path (Join-Path $InstallRoot 'jdk') $name)
    if (-not (Test-JdkHome $dest)) { throw "JDK install did not validate: $dest" }
    Write-Ok "JDK 17 installed: $dest"
    return $dest
}

function Install-Gradle {
    $zip = Get-CheckedDownload `
        "gradle-$GradleVersion-bin.zip" `
        "https://services.gradle.org/distributions/gradle-$GradleVersion-bin.zip" `
        137393837 `
        'bd71102213493060956ec229d946beee57158dbd89d0e62b91bca0fa2c5f3531' `
        'SHA256'
    $top = Expand-ArchiveToStaging $zip 'gradle'
    $name = Split-Path -Leaf $top
    $dest = Publish-Directory $top (Join-Path (Join-Path $InstallRoot 'gradle') $name)
    if (-not (Test-GradleHome $dest)) { throw "Gradle install did not validate: $dest" }
    Write-Ok "Gradle $GradleVersion installed: $dest"
    return $dest
}

function Install-Ndk {
    $zip = Get-CheckedDownload `
        'android-ndk-r27c-windows.zip' `
        'https://dl.google.com/android/repository/android-ndk-r27c-windows.zip' `
        781511249 `
        'AC5F7762764B1F15341094E148AD4F847D050C38' `
        'SHA1'
    $top = Expand-ArchiveToStaging $zip 'ndk'
    $dest = Publish-Directory $top (Join-Path (Join-Path $InstallRoot 'ndk') "android-ndk-$NdkRelease")
    if (-not (Test-NdkRoot $dest)) { throw "NDK install did not validate: $dest" }
    Write-Ok "Android NDK $NdkRelease ($NdkVersion) installed: $dest"
    return $dest
}

function Install-CmdlineTools {
    param([string]$SdkRoot)

    $zip = Get-CheckedDownload `
        'commandlinetools-win-15859902_latest.zip' `
        'https://dl.google.com/android/repository/commandlinetools-win-15859902_latest.zip' `
        155655386 `
        '90AE805D20434428BFFCB699C290860F19BB5F66A67E6B330067E3DE801FB04A' `
        'SHA256'
    $top = Expand-ArchiveToStaging $zip 'cmdline-tools'
    $dest = Publish-Directory $top (Join-Path (Join-Path $SdkRoot 'cmdline-tools') 'latest')
    $manager = Join-Path $dest 'bin\sdkmanager.bat'
    if (-not (Test-Path -LiteralPath $manager)) {
        throw "sdkmanager.bat was not found after installing command-line tools: $dest"
    }
    Write-Ok "Android command-line tools installed: $dest"
    return $manager
}

function Invoke-SdkManager {
    param([string]$Manager, [string[]]$Arguments, [string]$Label)
    Write-Host "   -> $Label" -ForegroundColor Gray
    & $Manager @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed (exit $LASTEXITCODE)"
    }
}

function Install-SdkPackages {
    param([string]$SdkRoot, [string]$Manager, [string]$JdkHome)

    if ([string]::IsNullOrWhiteSpace($JdkHome)) {
        throw 'JDK 17 is required before Android SDK packages can be installed.'
    }
    $oldJavaHome = $env:JAVA_HOME
    $oldPath = $env:PATH
    $env:JAVA_HOME = $JdkHome
    # sdkmanager.bat calls findstr and java through PATH. A stripped-down
    # PowerShell host can omit System32, which makes its JDK check fail even
    # when JAVA_HOME is correct, so add the Windows directories explicitly.
    $env:PATH = (Join-Path $JdkHome 'bin') + ';' + (Join-Path $env:SystemRoot 'System32') + ';' + $env:SystemRoot + ';' + $oldPath
    try {
        # sdkmanager asks once per license and needs more answers than the
        # number of packages. Feed a file through cmd.exe: a PowerShell
        # pipeline closed after a fixed number of lines leaves it waiting.
        $answerFile = Join-Path $env:TEMP ("pb64-sdk-licenses-{0}.txt" -f [Guid]::NewGuid().ToString('N'))
        [IO.File]::WriteAllText($answerFile, ("y`r`n" * 500))
        Write-Host '   -> sdkmanager --licenses' -ForegroundColor Gray
        $cmdExe = Join-Path $env:SystemRoot 'System32\cmd.exe'
        & $cmdExe /c "`"$Manager`" --sdk_root=`"$SdkRoot`" --licenses < `"$answerFile`""
        $licenseRc = $LASTEXITCODE
        Remove-Item -LiteralPath $answerFile -Force -ErrorAction SilentlyContinue
        if ($licenseRc -ne 0) {
            throw "sdkmanager --licenses failed (exit $licenseRc)"
        }

        $packages = @(
            'platform-tools',
            'platforms;android-34',
            'platforms;android-35',
            'build-tools;34.0.0',
            'build-tools;35.0.0'
        )
        $arguments = @("--sdk_root=$SdkRoot") + $packages
        Invoke-SdkManager $Manager $arguments 'sdkmanager install platform-tools, platforms 34/35, build-tools 34/35'
    } finally {
        $env:JAVA_HOME = $oldJavaHome
        $env:PATH = $oldPath
    }
}

function Set-UserEnvironment {
    param(
        [string]$JdkHome,
        [string]$GradleHome,
        [string]$SdkRoot,
        [string]$NdkRoot,
        [string]$HomeRoot,
        [string]$Manager
    )

    $values = [ordered]@{
        'JAVA_HOME'          = $JdkHome
        'GRADLE_HOME'        = $GradleHome
        'ANDROID_HOME'       = $SdkRoot
        'ANDROID_SDK_ROOT'   = $SdkRoot
        'ANDROID_NDK_HOME'   = $NdkRoot
        'ANDROID_NDK_ROOT'   = $NdkRoot
        'PB64_ANDROID_HOME'  = $HomeRoot
    }
    foreach ($name in $values.Keys) {
        $value = $values[$name]
        if ([string]::IsNullOrWhiteSpace($value)) { continue }
        $current = [Environment]::GetEnvironmentVariable($name, 'User')
        if ($current -ne $value) {
            [Environment]::SetEnvironmentVariable($name, $value, 'User')
            Write-Ok "$name = $value"
        } else {
            Write-Host "   [=] $name already set" -ForegroundColor DarkGray
        }
    }

    $add = @()
    if (-not [string]::IsNullOrWhiteSpace($JdkHome)) { $add += (Join-Path $JdkHome 'bin') }
    if (-not [string]::IsNullOrWhiteSpace($GradleHome)) { $add += (Join-Path $GradleHome 'bin') }
    if (-not [string]::IsNullOrWhiteSpace($SdkRoot)) {
        $add += (Join-Path $SdkRoot 'platform-tools')
        if (-not [string]::IsNullOrWhiteSpace($Manager)) {
            $add += (Split-Path -Parent $Manager)
        }
    }
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ([string]::IsNullOrWhiteSpace($userPath)) { $userPath = '' }
    $parts = @($userPath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    $changed = $false
    foreach ($entry in $add) {
        $normalized = $entry.TrimEnd('\')
        $exists = $false
        foreach ($part in $parts) {
            if ($part.TrimEnd('\') -ieq $normalized) { $exists = $true; break }
        }
        if (-not $exists) {
            $parts += $entry
            $changed = $true
            Write-Ok "PATH += $entry"
        }
    }
    if ($changed) {
        [Environment]::SetEnvironmentVariable('Path', ($parts -join ';'), 'User')
    }
}

# ---------- Detection ----------
$roots = @($InstallRoot)
if (-not $RootExplicit -and -not [string]::IsNullOrWhiteSpace($env:PB64_ANDROID_HOME)) {
    if ($env:PB64_ANDROID_HOME -ne $InstallRoot) { $roots += $env:PB64_ANDROID_HOME }
}

$JdkHome = Find-JdkHome $roots
$GradleHome = Find-GradleHome $roots
$SdkRoot = Find-SdkRoot $roots
$NdkRoot = Find-NdkRoot $roots $SdkRoot
$SdkManager = Find-SdkManager $SdkRoot

Write-Host ''
Write-Host 'PB64Basic Android build toolchain' -ForegroundColor White
Write-Host "Install root: $InstallRoot" -ForegroundColor Gray
Write-Host ("  JDK 17.0.21+9     : {0}" -f $(if ($JdkHome) { $JdkHome } else { 'MISSING' }))
Write-Host ("  Gradle 8.14.3     : {0}" -f $(if ($GradleHome) { $GradleHome } else { 'MISSING' }))
Write-Host ("  Android SDK       : {0}" -f $(if ($SdkRoot) { $SdkRoot } else { 'MISSING' }))
if ($SdkRoot) {
    Write-Host ("    platform-tools  : {0}" -f $(if (Test-Path -LiteralPath (Join-Path $SdkRoot 'platform-tools\adb.exe')) { 'present' } else { 'MISSING' }))
    Write-Host ("    platforms 34/35 : {0}" -f $(if ((Test-Path -LiteralPath (Join-Path $SdkRoot 'platforms\android-34\android.jar')) -and (Test-Path -LiteralPath (Join-Path $SdkRoot 'platforms\android-35\android.jar'))) { 'present' } else { 'MISSING' }))
    Write-Host ("    build-tools 34/35: {0}" -f $(if ((Test-Path -LiteralPath (Join-Path $SdkRoot 'build-tools\34.0.0\aapt2.exe')) -and (Test-Path -LiteralPath (Join-Path $SdkRoot 'build-tools\35.0.0\aapt2.exe'))) { 'present' } else { 'MISSING' }))
}
Write-Host ("  Android NDK r27c  : {0}" -f $(if ($NdkRoot) { $NdkRoot } else { 'MISSING' }))
Write-Host ("  sdkmanager        : {0}" -f $(if ($SdkManager) { $SdkManager } else { 'MISSING' }))

$missing = @()
if (-not $JdkHome) { $missing += 'JDK 17.0.21+9' }
if (-not $GradleHome) { $missing += 'Gradle 8.14.3' }
if (-not (Test-SdkComplete $SdkRoot)) { $missing += 'Android SDK packages' }
if (-not $SkipNdk -and -not $NdkRoot) { $missing += "Android NDK $NdkRelease" }

if ($CheckOnly) {
    Write-Host ''
    if ($missing.Count -eq 0) {
        Write-Ok 'All required Android build components are present.'
    } else {
        Write-Warn ("Missing: " + ($missing -join ', '))
        Write-Host 'Re-run without -CheckOnly to install them.' -ForegroundColor Gray
    }
    exit 0
}

# ---------- Install ----------
if (-not (Test-Path -LiteralPath $InstallRoot)) {
    New-Item -ItemType Directory -Path $InstallRoot -Force | Out-Null
}
if (-not (Test-Path -LiteralPath $DownloadDir)) {
    New-Item -ItemType Directory -Path $DownloadDir -Force | Out-Null
}

Write-Step '1/4  JDK 17 (Temurin 17.0.21+9)'
if ($JdkHome) {
    Write-Ok "Already present: $JdkHome"
} else {
    $JdkHome = Install-Jdk
}

Write-Step '2/4  Gradle 8.14.3'
if ($GradleHome) {
    Write-Ok "Already present: $GradleHome"
} else {
    $GradleHome = Install-Gradle
}

Write-Step "3/4  Android NDK $NdkRelease ($NdkVersion)"
if ($SkipNdk) {
    Write-Warn 'Skipped by -SkipNdk. APK builds need this NDK.'
} elseif ($NdkRoot) {
    Write-Ok "Already present: $NdkRoot"
} else {
    $NdkRoot = Install-Ndk
}

Write-Step '4/4  Android SDK'
if ($SdkRoot -and $SdkManager -and (Test-SdkComplete $SdkRoot)) {
    Write-Ok "SDK packages already present: $SdkRoot"
} else {
    if ([string]::IsNullOrWhiteSpace($SdkRoot)) {
        $SdkRoot = Join-Path $InstallRoot 'Sdk'
    }
    if (-not (Test-Path -LiteralPath $SdkRoot)) {
        New-Item -ItemType Directory -Path $SdkRoot -Force | Out-Null
    }
    if ([string]::IsNullOrWhiteSpace($SdkManager)) {
        $SdkManager = Install-CmdlineTools $SdkRoot
    } else {
        Write-Ok "sdkmanager already present: $SdkManager"
    }
    if (Test-SdkComplete $SdkRoot) {
        Write-Ok "SDK packages already present: $SdkRoot"
    } elseif ($SkipSdkPackages) {
        Write-Warn 'SDK package installation skipped by -SkipSdkPackages.'
    } else {
        Install-SdkPackages $SdkRoot $SdkManager $JdkHome
    }
}

# ---------- Verify ----------
Write-Step 'Verifying installed toolchain'
$errors = @()
if (-not (Test-JdkHome $JdkHome)) { $errors += "JDK invalid: $JdkHome" }
if (-not (Test-GradleHome $GradleHome)) { $errors += "Gradle invalid: $GradleHome" }
if (-not $SkipNdk -and -not (Test-NdkRoot $NdkRoot)) { $errors += "NDK invalid: $NdkRoot" }
if (-not (Test-SdkComplete $SdkRoot)) { $errors += "Android SDK incomplete: $SdkRoot" }
if ($errors.Count -gt 0) {
    foreach ($e in $errors) { Write-Fail $e }
    throw 'Android toolchain verification failed.'
}
Write-Ok 'JDK, Gradle, Android SDK and NDK all validate.'

if (-not [string]::IsNullOrWhiteSpace($NdkRoot)) {
    $currentNdk = $env:ANDROID_NDK_HOME
    if ($currentNdk -and ($currentNdk -ne $NdkRoot) -and (Test-NdkRoot $currentNdk)) {
        Write-Warn "Another NDK is already active: $currentNdk"
    }
}

# ---------- Environment ----------
if ($NoEnv) {
    Write-Warn 'Environment variables were not changed (-NoEnv).'
} else {
    Write-Step 'Updating user environment'
    Set-UserEnvironment $JdkHome $GradleHome $SdkRoot $NdkRoot $InstallRoot $SdkManager
    Write-Host '   Open a new terminal (or sign out and back in) before building APKs.' -ForegroundColor Gray
}

# ---------- Summary ----------
Write-Step 'Android toolchain summary'
Write-Ok "JDK      : $JdkHome"
Write-Ok "Gradle   : $GradleHome"
Write-Ok "SDK      : $SdkRoot"
Write-Ok "NDK      : $NdkRoot"
if ($SdkManager) { Write-Ok "sdkmanager: $SdkManager" }
Write-Host ''
Write-Host 'Build an APK with:' -ForegroundColor White
Write-Host '   powershell -NoProfile -ExecutionPolicy Bypass -File tools\build_apk.ps1 "Welcome Hello.bas"' -ForegroundColor Gray
Write-Host ''
Write-Host 'Android toolchain setup finished.' -ForegroundColor Cyan
