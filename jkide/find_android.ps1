<#
  find_android.ps1 - locate the Android build software root and emit
  [Compilers] key=value lines for JKIDE.ini.

  Why this exists
  ---------------
  setup.bat used to know nothing about Android.  The Android APK menu in
  JK-IDE reads AndroidRoot / AndroidBuildScript / AndroidSdk / AndroidNdk /
  AndroidJdk / AndroidGradle / AndroidBaseline from JKIDE.ini.  Hard-coding
  D:\AndroidToolchain works on this machine but not on a company laptop.
  This locator probes the usual roots, validates the four components and
  keeps the previous INI values when a re-run cannot find them.

  Output: only key=value lines (no headings).  Exit 0 when an Android root
  or build script was found, exit 1 otherwise.
#>
param(
    [string]$RepoRoot = '',
    [string]$ExistingIni = '',
    [string]$AndroidRoot = '',
    [string]$UserProfile = $env:USERPROFILE
)

# setup.bat used to pass "%~dp0" (a path ending in a backslash).  Windows
# PowerShell parses the closing quote after that backslash inconsistently, so
# RepoRoot arrived empty or truncated and every RepoRoot-derived key was
# written blank.  Normalise both the batch caller and direct invocations.
$RepoRoot = $RepoRoot.Trim().Trim('"').TrimEnd('\')

$ErrorActionPreference = 'SilentlyContinue'

function Test-Dir {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { return $false }
    return (Test-Path -LiteralPath $Path -PathType Container)
}

function Test-File {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { return $false }
    return (Test-Path -LiteralPath $Path -PathType Leaf)
}

function Read-IniValues {
    param([string]$Path)
    $values = @{}
    if (-not (Test-File $Path)) { return $values }
    $section = ''
    foreach ($line in Get-Content -LiteralPath $Path -ErrorAction SilentlyContinue) {
        $text = $line.Trim()
        if ([string]::IsNullOrWhiteSpace($text)) { continue }
        if ($text.StartsWith(';')) { continue }
        if ($text -match '^\s*\[(.+)\]\s*$') {
            $section = $matches[1].Trim()
            continue
        }
        if ($section -ieq 'Compilers' -and $text -match '^([^=]+)=(.*)$') {
            $key = $matches[1].Trim().ToLowerInvariant()
            $value = $matches[2].Trim().Trim('"')
            if ($key) { $values[$key] = $value }
        }
    }
    return $values
}

function Get-LatestDir {
    param([string]$Base, [scriptblock]$Test)
    if (-not (Test-Dir $Base)) { return '' }
    $dirs = Get-ChildItem -LiteralPath $Base -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending
    foreach ($dir in $dirs) {
        if (& $Test $dir.FullName) { return $dir.FullName }
    }
    return ''
}

function Find-JdkHome {
    param([string]$Root)
    return (Get-LatestDir (Join-Path $Root 'jdk') {
        param($Dir)
        (Test-File (Join-Path $Dir 'bin\java.exe')) -and
        (Test-File (Join-Path $Dir 'release'))
    })
}

function Find-GradleHome {
    param([string]$Root)
    return (Get-LatestDir (Join-Path $Root 'gradle') {
        param($Dir)
        (Test-File (Join-Path $Dir 'bin\gradle.bat')) -and
        (Test-File (Join-Path $Dir 'lib\gradle-launcher-8.14.3.jar'))
    })
}

function Find-SdkRoot {
    param([string]$Root)
    foreach ($name in @('Sdk', 'sdk', 'AndroidSdk', 'android-sdk')) {
        $sdk = Join-Path $Root $name
        if (-not (Test-Dir $sdk)) { continue }
        if ((Test-File (Join-Path $sdk 'platform-tools\adb.exe')) -or
            (Test-Dir (Join-Path $sdk 'platforms')) -or
            (Test-Dir (Join-Path $sdk 'build-tools'))) {
            return $sdk
        }
    }
    return ''
}

function Find-NdkRoot {
    param([string]$Root, [string]$SdkRoot)
    $bases = @((Join-Path $Root 'ndk'))
    if ($SdkRoot) { $bases += (Join-Path $SdkRoot 'ndk') }
    foreach ($base in $bases) {
        $ndk = Get-LatestDir $base {
            param($Dir)
            (Test-File (Join-Path $Dir 'toolchains\llvm\prebuilt\windows-x86_64\bin\clang.exe')) -and
            (Test-Dir (Join-Path $Dir 'toolchains\llvm\prebuilt\windows-x86_64\sysroot'))
        }
        if ($ndk) { return $ndk }
    }
    return ''
}

function Resolve-AndroidRoot {
    param([string]$Root)

    if ([string]::IsNullOrWhiteSpace($Root)) { return $null }
    $root = $Root.Trim().Trim('"').TrimEnd('\')
    if (-not (Test-Dir $root)) { return $null }

    $jdk = Find-JdkHome $root
    $gradle = Find-GradleHome $root
    $sdk = Find-SdkRoot $root
    $ndk = Find-NdkRoot $root $sdk
    $baseline = ''
    if (Test-Dir (Join-Path $root 'baseline')) { $baseline = Join-Path $root 'baseline' }

    $score = 0
    if ($jdk) { $score++ }
    if ($gradle) { $score++ }
    if ($sdk) { $score++ }
    if ($ndk) { $score++ }

    # A usable APK build root needs the SDK and NDK at minimum.  The JDK and
    # Gradle may live elsewhere, but setup-android.ps1 installs all four.
    if (-not $sdk -or -not $ndk) { return $null }
    if ($score -lt 3) { return $null }

    return [pscustomobject]@{
        Root     = $root
        Jdk      = $jdk
        Gradle   = $gradle
        Sdk      = $sdk
        Ndk      = $ndk
        Baseline = $baseline
        Score    = $score
    }
}

function Find-BuildScript {
    param([string]$Repo)

    $candidates = New-Object System.Collections.ArrayList
    $add = {
        param([string]$Path)
        if ([string]::IsNullOrWhiteSpace($Path)) { return }
        $path = $Path.Trim().Trim('"')
        if (Test-File $path) { $null = $candidates.Add($path) }
    }

    if (-not [string]::IsNullOrWhiteSpace($Repo)) {
        & $add (Join-Path $Repo 'PB64Basic-Android\tools\build_apk.ps1')
        & $add (Join-Path $Repo '..\PB64Basic-Android\tools\build_apk.ps1')
        & $add (Join-Path $Repo '..\..\PB64Basic-Android\tools\build_apk.ps1')
        & $add (Join-Path $Repo 'tools\build_apk.ps1')
        & $add (Join-Path $Repo '..\tools\build_apk.ps1')
        $parent = Join-Path $Repo '..'
        foreach ($dir in Get-ChildItem -LiteralPath $parent -Directory -Filter 'PB64Basic-Android' -ErrorAction SilentlyContinue) {
            & $add (Join-Path $dir.FullName 'tools\build_apk.ps1')
        }

        # A release zip is often extracted several levels below the sibling
        # PB64Basic-Android checkout (for example
        # new-chat\powerbasilisk_test\pbsrc_b378\target\smoke\PB64Basic-Enhanced-...).
        # Walk the ancestors so setup.bat still finds that checkout instead of
        # writing an empty AndroidBuildScript.
        $cursor = [IO.Path]::GetFullPath($Repo).TrimEnd('\')
        for ($depth = 0; $depth -lt 8; $depth++) {
            $up = Split-Path -Parent $cursor
            if ([string]::IsNullOrWhiteSpace($up) -or $up -eq $cursor) { break }
            $cursor = $up
            & $add (Join-Path $cursor 'PB64Basic-Android\tools\build_apk.ps1')
            & $add (Join-Path $cursor 'tools\build_apk.ps1')
        }
    }
    if ($env:PB64_ANDROID_PROJECT) {
        & $add (Join-Path $env:PB64_ANDROID_PROJECT 'tools\build_apk.ps1')
    }

    if ($candidates.Count -eq 0) { return '' }

    # Prefer the script closest to setup.bat (repo\PB64Basic-Android) over a
    # deeper archive copy.  Get-ChildItem order is already shallow-first.
    $path = $candidates[0]
    $resolved = Resolve-Path -LiteralPath $path -ErrorAction SilentlyContinue
    if ($resolved) { return $resolved.Path }
    return $path
}

$old = Read-IniValues $ExistingIni

$roots = New-Object System.Collections.ArrayList
$seen = @{}
$addRoot = {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { return }
    $path = $Path.Trim().Trim('"').TrimEnd('\')
    if (-not (Test-Dir $path)) { return }
    $key = $path.ToLowerInvariant()
    if ($seen.ContainsKey($key)) { return }
    $seen[$key] = $true
    $null = $roots.Add($path)
}

& $addRoot $AndroidRoot
& $addRoot $env:PB64_ANDROID_HOME
if ($UserProfile) { & $addRoot (Join-Path $UserProfile 'AndroidToolchain') }
if ($env:LOCALAPPDATA) { & $addRoot (Join-Path $env:LOCALAPPDATA 'PB64Basic\AndroidToolchain') }

foreach ($drive in Get-PSDrive -PSProvider FileSystem) {
    if ($drive.Root -notmatch '^[A-Za-z]:\\$') { continue }
    if (-not (Test-Dir $drive.Root)) { continue }
    foreach ($rel in @(
        'AndroidToolchain',
        'Android',
        'Software\AndroidToolchain',
        'Software\Android',
        'Tools\AndroidToolchain',
        'Tools\Android'
    )) {
        & $addRoot (Join-Path $drive.Root $rel)
    }
}

$resolved = New-Object System.Collections.ArrayList
foreach ($root in $roots) {
    $item = Resolve-AndroidRoot $root
    if ($item) { $null = $resolved.Add($item) }
}

$selected = $null
if ($resolved.Count -gt 0) {
    $selected = $resolved | Sort-Object Score -Descending | Select-Object -First 1
}

# If probing failed this run, trust the previous INI values only when they
# still point at real files/directories.  This is the same no-blanking rule
# setup.bat uses for Pb64Exe / Pbxb64Exe.
$oldRoot = ''
if ($old.ContainsKey('androidroot')) { $oldRoot = $old['androidroot'] }
if (-not $selected -and $oldRoot) {
    $selected = Resolve-AndroidRoot $oldRoot
}

function Pick-Dir {
    param([string]$NewValue, [string]$OldKey)
    if (-not [string]::IsNullOrWhiteSpace($NewValue) -and (Test-Dir $NewValue)) { return $NewValue }
    if ($old.ContainsKey($OldKey) -and (Test-Dir $old[$OldKey])) { return $old[$OldKey] }
    return ''
}

function Pick-File {
    param([string]$NewValue, [string]$OldKey)
    if (-not [string]::IsNullOrWhiteSpace($NewValue) -and (Test-File $NewValue)) { return $NewValue }
    if ($old.ContainsKey($OldKey) -and (Test-File $old[$OldKey])) { return $old[$OldKey] }
    return ''
}

$buildScript = Find-BuildScript $RepoRoot
$buildScript = Pick-File $buildScript 'androidbuildscript'

$rootValue = ''
$sdkValue = ''
$ndkValue = ''
$jdkValue = ''
$gradleValue = ''
$baselineValue = ''
if ($selected) {
    $rootValue = $selected.Root
    $sdkValue = $selected.Sdk
    $ndkValue = $selected.Ndk
    $jdkValue = $selected.Jdk
    $gradleValue = ''
    if ($selected.Gradle) { $gradleValue = Join-Path $selected.Gradle 'bin\gradle.bat' }
    $baselineValue = $selected.Baseline
}
$rootValue = Pick-Dir $rootValue 'androidroot'
$sdkValue = Pick-Dir $sdkValue 'androidsdk'
$ndkValue = Pick-Dir $ndkValue 'androidndk'
$jdkValue = Pick-Dir $jdkValue 'androidjdk'
$gradleValue = Pick-File $gradleValue 'androidgradle'
$baselineValue = Pick-Dir $baselineValue 'androidbaseline'

$pb64Value = ''
if (-not [string]::IsNullOrWhiteSpace($RepoRoot)) {
    # A release zip has PB64Basic.exe at its root; a source checkout has it
    # under target\release.  Check the release layout first so a fresh install
    # never writes an empty AndroidPb64Exe.
    $candidate = Join-Path $RepoRoot 'PB64Basic.exe'
    if (Test-File $candidate) { $pb64Value = $candidate }
    if (-not $pb64Value) {
        $candidate = Join-Path $RepoRoot 'target\release\PB64Basic.exe'
        if (Test-File $candidate) { $pb64Value = $candidate }
    }
}
if (-not $pb64Value -and $old.ContainsKey('androidpb64exe') -and (Test-File $old['androidpb64exe'])) {
    $pb64Value = $old['androidpb64exe']
}
if (-not $pb64Value -and $old.ContainsKey('pb64exe') -and (Test-File $old['pb64exe'])) {
    $pb64Value = $old['pb64exe']
}

Write-Output ('AndroidRoot=' + $rootValue)
Write-Output ('AndroidBuildScript=' + $buildScript)
Write-Output ('AndroidSdk=' + $sdkValue)
Write-Output ('AndroidNdk=' + $ndkValue)
Write-Output ('AndroidJdk=' + $jdkValue)
Write-Output ('AndroidGradle=' + $gradleValue)
Write-Output ('AndroidBaseline=' + $baselineValue)
Write-Output ('AndroidPb64Exe=' + $pb64Value)

if ($rootValue -or $buildScript) { exit 0 }
exit 1
