param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Source,

    [string]$AppName = '',

    [string]$OutputDir = '',

    [string]$ApplicationId = '',

    [string]$VersionName = '',

    [string]$OutputName = '',

    [ValidateSet('arm64-v8a', 'x86_64', 'both')]
    [string]$Abi = 'both',

    [string]$PB64Basic = '',

    # Explicit path to pbcompiler\runtime\pb_runtime.c.  When omitted, the
    # builder derives it from the compiler executable's repository root.
    [string]$RuntimeSource = '',

    [string]$Ndk = '',

    [string]$Gradle = '',

    [string]$JavaHome = '',

    [string]$AndroidSdk = '',

    [string]$RuntimeCache = '',

    [switch]$RebuildRuntime,

    [switch]$SkipGradle,

    [switch]$Install,

    [switch]$Run
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ScriptDir = $PSScriptRoot
$ProjectRoot = Split-Path -Parent $ScriptDir
$TemplateRoot = Join-Path $ProjectRoot 'PBApp'

function Resolve-FirstPath {
    param(
        [string]$Label,
        [string[]]$Candidates
    )
    foreach ($candidate in $Candidates) {
        if ([string]::IsNullOrWhiteSpace($candidate)) { continue }
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }
    throw "$Label not found. Checked: $($Candidates -join ', ')"
}

function Invoke-Native {
    param(
        [string]$Exe,
        [string[]]$Arguments,
        [string]$Label
    )
    Write-Host "  -> $Label"
    & $Exe @Arguments
    $rc = $LASTEXITCODE
    if ($rc -ne 0) {
        throw "$Label failed (exit $rc)"
    }
}

if (-not (Test-Path -LiteralPath $TemplateRoot)) {
    throw "PBApp template not found: $TemplateRoot"
}

$Source = (Resolve-Path -LiteralPath $Source).Path
if ([IO.Path]::GetExtension($Source).ToLowerInvariant() -ne '.bas') {
    throw "Source must be a .bas file: $Source"
}

if ([string]::IsNullOrWhiteSpace($AppName)) {
    $AppName = [IO.Path]::GetFileNameWithoutExtension($Source)
}
$AppName = $AppName.Trim()
if ($AppName -match "[`r`n]") {
    throw 'AppName must be a single line'
}
if ([string]::IsNullOrWhiteSpace($AppName)) {
    $AppName = 'PB64Basic App'
}

$ExpectedPackage = if ([string]::IsNullOrWhiteSpace($ApplicationId)) {
    'com.example.pb64app'
} else {
    $ApplicationId.Trim()
}
if ($ExpectedPackage -notmatch '^[A-Za-z][A-Za-z0-9_]*(\.[A-Za-z][A-Za-z0-9_]*)+$') {
    throw "ApplicationId is not a valid Android package name: $ExpectedPackage"
}

$Slug = [regex]::Replace($AppName, '[^\p{L}\p{N}._-]+', '-').Trim('-', '.')
if ([string]::IsNullOrWhiteSpace($Slug)) { $Slug = 'pbapp' }

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $ProjectRoot 'dist'
}
$OutputDir = [IO.Path]::GetFullPath($OutputDir)
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

$PB64Basic = Resolve-FirstPath 'PB64Basic.exe' @(
    $PB64Basic,
    (Join-Path $ProjectRoot '..\PB64Basic.exe'),
    (Join-Path $ProjectRoot '..\powerbasilisk_test\pbsrc_b378\target\release\PB64Basic.exe')
)

# Walk up from target\release\PB64Basic.exe until the compiler repository root
# (the directory that contains pbcompiler\runtime\pb_runtime.c) is found.  This
# keeps the Android builder usable from any checkout location instead of a
# hard-coded C:\Users\... path.
$CompilerRoot = Split-Path -Parent $PB64Basic
while ($CompilerRoot -and
       -not (Test-Path -LiteralPath (Join-Path $CompilerRoot 'pbcompiler\runtime\pb_runtime.c'))) {
    $parent = Split-Path -Parent $CompilerRoot
    if ([string]::IsNullOrWhiteSpace($parent) -or $parent -eq $CompilerRoot) {
        $CompilerRoot = ''
        break
    }
    $CompilerRoot = $parent
}

$RuntimeSource = Resolve-FirstPath 'PB runtime source' @(
    $RuntimeSource,
    $(if ($CompilerRoot) { Join-Path $CompilerRoot 'pbcompiler\runtime\pb_runtime.c' }),
    $(if ($env:PB64_RUNTIME_SOURCE) { $env:PB64_RUNTIME_SOURCE }),
    (Join-Path $ProjectRoot '..\powerbasilisk_test\pbsrc_b378\pbcompiler\runtime\pb_runtime.c')
)

if ([string]::IsNullOrWhiteSpace($VersionName)) {
    $versionCandidates = @(
        $(if ($CompilerRoot) { Join-Path $CompilerRoot 'VERSION' }),
        (Join-Path $ProjectRoot '..\powerbasilisk_test\pbsrc_b378\VERSION')
    )
    foreach ($versionFile in $versionCandidates) {
        if ($versionFile -and (Test-Path -LiteralPath $versionFile)) {
            $VersionName = (Get-Content -LiteralPath $versionFile -Raw).Trim()
            if (-not [string]::IsNullOrWhiteSpace($VersionName)) { break }
        }
    }
    if ([string]::IsNullOrWhiteSpace($VersionName)) {
        $VersionName = '1.0'
    }
}

Write-Host "[env] PB64Basic: $PB64Basic"
Write-Host "[env] runtime  : $RuntimeSource"
Write-Host "[env] version  : $VersionName"

$Ndk = Resolve-FirstPath 'Android NDK' @(
    $Ndk,
    $env:ANDROID_NDK_HOME,
    $env:ANDROID_NDK_ROOT,
    $(if ($env:PB64_ANDROID_HOME) { Join-Path $env:PB64_ANDROID_HOME 'ndk\android-ndk-r27c' }),
    $(if ($env:LOCALAPPDATA) { Join-Path $env:LOCALAPPDATA 'PB64Basic\AndroidToolchain\ndk\android-ndk-r27c' }),
    'D:\AndroidToolchain\ndk\android-ndk-r27c'
)
$Clang = Join-Path $Ndk 'toolchains\llvm\prebuilt\windows-x86_64\bin\clang.exe'
$Sysroot = Join-Path $Ndk 'toolchains\llvm\prebuilt\windows-x86_64\sysroot'
if (-not (Test-Path -LiteralPath $Clang)) { throw "NDK clang not found: $Clang" }
if (-not (Test-Path -LiteralPath $Sysroot)) { throw "NDK sysroot not found: $Sysroot" }

$Gradle = Resolve-FirstPath 'Gradle' @(
    $Gradle,
    $(if ($env:GRADLE_HOME) { Join-Path $env:GRADLE_HOME 'bin\gradle.bat' }),
    $(if ($env:PB64_ANDROID_HOME) { Join-Path $env:PB64_ANDROID_HOME 'gradle\gradle-8.14.3\bin\gradle.bat' }),
    $(if ($env:LOCALAPPDATA) { Join-Path $env:LOCALAPPDATA 'PB64Basic\AndroidToolchain\gradle\gradle-8.14.3\bin\gradle.bat' }),
    'D:\AndroidToolchain\gradle\gradle-8.14.3\bin\gradle.bat'
)
$JavaHome = Resolve-FirstPath 'JDK 17' @(
    $JavaHome,
    $env:JAVA_HOME,
    $(if ($env:PB64_ANDROID_HOME) { Join-Path $env:PB64_ANDROID_HOME 'jdk\jdk-17.0.21+9' }),
    $(if ($env:LOCALAPPDATA) { Join-Path $env:LOCALAPPDATA 'PB64Basic\AndroidToolchain\jdk\jdk-17.0.21+9' }),
    'D:\AndroidToolchain\jdk\jdk-17.0.21+9'
)

if ([string]::IsNullOrWhiteSpace($AndroidSdk)) {
    # Older shells can keep a stale ANDROID_HOME after the SDK moves.
    # Resolve-FirstPath skips any candidate that no longer exists, so a
    # dead C: path falls through to ANDROID_SDK_ROOT and then the D: default.
    $AndroidSdk = Resolve-FirstPath 'Android SDK' @(
        $env:ANDROID_HOME,
        $env:ANDROID_SDK_ROOT,
        $(if ($env:PB64_ANDROID_HOME) { Join-Path $env:PB64_ANDROID_HOME 'Sdk' }),
        $(if ($env:LOCALAPPDATA) { Join-Path $env:LOCALAPPDATA 'PB64Basic\AndroidToolchain\Sdk' }),
        'D:\AndroidToolchain\Sdk'
    )
} else {
    # An explicitly supplied -AndroidSdk is an operator choice: require it.
    $AndroidSdk = (Resolve-Path -LiteralPath $AndroidSdk).Path
}
if (-not (Test-Path -LiteralPath $AndroidSdk -PathType Container)) {
    throw "Android SDK is not a directory: $AndroidSdk"
}
Write-Host "[env] Android SDK: $AndroidSdk"

if ([string]::IsNullOrWhiteSpace($RuntimeCache)) {
    if ($env:PB64_ANDROID_HOME) {
        $RuntimeCache = Join-Path $env:PB64_ANDROID_HOME 'baseline'
    } elseif ($env:LOCALAPPDATA) {
        $RuntimeCache = Join-Path $env:LOCALAPPDATA 'PB64Basic\AndroidToolchain\baseline'
    } else {
        $RuntimeCache = 'D:\AndroidToolchain\baseline'
    }
}

$env:JAVA_HOME = $JavaHome
$env:ANDROID_HOME = $AndroidSdk
$env:ANDROID_SDK_ROOT = $AndroidSdk
$env:PATH = (Join-Path $JavaHome 'bin') + ';' + (Split-Path -Parent $Gradle) + ';' + $env:PATH

$AbiList = if ($Abi -eq 'both') { @('arm64-v8a', 'x86_64') } else { @($Abi) }
$AbiMap = @{
    'arm64-v8a' = @{ Triple = 'aarch64-linux-android21'; Runtime = 'pb_runtime_arm64_hidden.o' }
    'x86_64'    = @{ Triple = 'x86_64-linux-android21'; Runtime = 'pb_runtime_android_x86_64_hidden.o' }
}

$Work = Join-Path ([IO.Path]::GetTempPath()) ('pb64android_' + $Slug + '_' + [Guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory -Path $Work -Force | Out-Null

Write-Host ''
Write-Host "PB64Basic Android APK builder"
Write-Host "  source : $Source"
Write-Host "  app    : $AppName"
Write-Host "  package: $ExpectedPackage"
Write-Host "  abi    : $($AbiList -join ', ')"
Write-Host "  work   : $Work"
Write-Host ""

$SupportSources = @(
    (Join-Path $ProjectRoot 'pb_android_sleep.c'),
    (Join-Path $ProjectRoot 'pb_android_console.c'),
    (Join-Path $ProjectRoot 'pb_android_input.c'),
    (Join-Path $ProjectRoot 'pb_android_file.c'),
    (Join-Path $ProjectRoot 'pb_android_thread.c'),
    (Join-Path $ProjectRoot 'pb_android_socket.c'),
    (Join-Path $ProjectRoot 'pb_android_clipboard.c'),
    (Join-Path $ProjectRoot 'pb_android_system.c'),
    (Join-Path $TemplateRoot 'app\src\main\jni\pb_jni.c')
)
foreach ($support in $SupportSources) {
    if (-not (Test-Path -LiteralPath $support)) {
        throw "Android support source not found: $support"
    }
}

$JniLibsRoot = Join-Path $TemplateRoot 'app\src\main\jniLibs'
New-Item -ItemType Directory -Path $JniLibsRoot -Force | Out-Null
foreach ($existing in Get-ChildItem -LiteralPath $JniLibsRoot -Directory -ErrorAction SilentlyContinue) {
    if ($existing.Name -notin $AbiList) {
        Remove-Item -LiteralPath $existing.FullName -Recurse -Force
    }
}

$Artifacts = @()

foreach ($abiName in $AbiList) {
    $target = $AbiMap[$abiName]
    $triple = $target.Triple
    Write-Host "[$abiName] compiling PB source"

    $irBase = Join-Path $Work "$Slug-$abiName.obj"
    Invoke-Native $PB64Basic @(
        'build', $Source, '--emit-llvm', '-o', $irBase,
        '--target', $triple,
        '--include-dir', (Split-Path -Parent $Source)
    ) "PB64Basic -> $triple IR"

    $ll = [IO.Path]::ChangeExtension($irBase, '.ll')
    if (-not (Test-Path -LiteralPath $ll)) {
        throw "PB64Basic did not write expected IR: $ll"
    }
    $irText = [IO.File]::ReadAllText($ll)
    if ($irText -notmatch "target triple = `"$([regex]::Escape($triple))`"") {
        throw "IR target triple mismatch in $ll (expected $triple)"
    }

    $appObj = Join-Path $Work "$Slug-$abiName.pic.o"
    Invoke-Native $Clang @(
        "--target=$triple", "--sysroot=$Sysroot",
        '-fPIC', '-O2', '-Wno-override-module', '-c', $ll, '-o', $appObj
    ) "clang app object ($abiName)"

    $supportObjs = @()
    foreach ($support in $SupportSources) {
        $stem = [IO.Path]::GetFileNameWithoutExtension($support)
        $obj = Join-Path $Work "$stem-$abiName.o"
        Invoke-Native $Clang @(
            "--target=$triple", "--sysroot=$Sysroot",
            '-fPIC', '-O2', '-c', $support, '-o', $obj
        ) "clang $stem ($abiName)"
        $supportObjs += $obj
    }

    $runtimeObj = ''
    $prebuilt = if ([string]::IsNullOrWhiteSpace($RuntimeCache)) { '' } else {
        Join-Path $RuntimeCache $target.Runtime
    }
    if (-not $RebuildRuntime -and $prebuilt -and (Test-Path -LiteralPath $prebuilt)) {
        $prebuiltTime = (Get-Item -LiteralPath $prebuilt).LastWriteTimeUtc
        $sourceTime = (Get-Item -LiteralPath $RuntimeSource).LastWriteTimeUtc
        if ($prebuiltTime -ge $sourceTime) {
            $runtimeObj = (Resolve-Path -LiteralPath $prebuilt).Path
            Write-Host "  -> runtime object: $runtimeObj (prebuilt)"
        }
    }
    if (-not $runtimeObj) {
        $runtimeObj = Join-Path $Work "pb_runtime-$abiName.o"
        Invoke-Native $Clang @(
            "--target=$triple", "--sysroot=$Sysroot",
            '-fPIC', '-O2', '-ffunction-sections', '-fdata-sections',
            '-fvisibility=hidden', '-c', $RuntimeSource, '-o', $runtimeObj, '-w'
        ) "clang full runtime ($abiName)"
        if ($prebuilt) {
            $cacheDir = Split-Path -Parent $prebuilt
            New-Item -ItemType Directory -Path $cacheDir -Force | Out-Null
            Copy-Item -LiteralPath $runtimeObj -Destination $prebuilt -Force
            $runtimeObj = (Resolve-Path -LiteralPath $prebuilt).Path
            Write-Host "  -> runtime cache refreshed: $runtimeObj"
        }
    }

    $so = Join-Path $Work "libpbapp-$abiName.so"
    Invoke-Native $Clang (@(
        "--target=$triple", "--sysroot=$Sysroot",
        '-shared', '-o', $so,
        $appObj, $runtimeObj
    ) + $supportObjs + @(
        '-llog',
        '-lm',
        '-Wl,--gc-sections',
        '-Wl,--no-undefined',
        '-Wl,-soname,libpbapp.so'
    )) "link libpbapp.so ($abiName)"

    $jniDir = Join-Path $JniLibsRoot $abiName
    New-Item -ItemType Directory -Path $jniDir -Force | Out-Null
    Copy-Item -LiteralPath $so -Destination (Join-Path $jniDir 'libpbapp.so') -Force

    $Artifacts += [pscustomobject]@{
        Abi = $abiName
        So = $so
        Bytes = (Get-Item -LiteralPath $so).Length
    }
}

if ($SkipGradle) {
    Write-Host ''
    Write-Host 'SkipGradle set; native libraries are ready but no APK was built.'
    $Artifacts | Format-Table -AutoSize | Out-String | Write-Host
    exit 0
}

Write-Host ''
Write-Host '[gradle] assembleDebug'
$localProperties = Join-Path $TemplateRoot 'local.properties'
$sdkValue = $AndroidSdk.Replace('\', '\\').Replace(':', '\:')
[IO.File]::WriteAllText(
    $localProperties,
    "sdk.dir=$sdkValue`n",
    (New-Object System.Text.UTF8Encoding($false))
)
Invoke-Native $Gradle @(
    '-p', $TemplateRoot,
    'assembleDebug',
    '--no-daemon',
    "-PpbAppName=$AppName",
    "-PpbApplicationId=$ExpectedPackage",
    "-PpbVersionName=$VersionName"
) 'Gradle assembleDebug'

$apk = Join-Path $TemplateRoot 'app\build\outputs\apk\debug\app-debug.apk'
if (-not (Test-Path -LiteralPath $apk)) {
    throw "Gradle did not produce expected APK: $apk"
}

$outName = if ([string]::IsNullOrWhiteSpace($OutputName)) {
    $Slug + '-debug.apk'
} else {
    $OutputName.Trim()
}
if (-not $outName.EndsWith('.apk', [StringComparison]::OrdinalIgnoreCase)) {
    $outName += '.apk'
}
$outApk = Join-Path $OutputDir $outName
Copy-Item -LiteralPath $apk -Destination $outApk -Force

$buildTools = Get-ChildItem -LiteralPath (Join-Path $AndroidSdk 'build-tools') -Directory |
    Sort-Object Name -Descending
$aapt = $null
$aapt2 = $null
$zipalign = $null
$apksigner = $null
foreach ($bt in $buildTools) {
    $aaptCandidate = Join-Path $bt.FullName 'aapt.exe'
    $aapt2Candidate = Join-Path $bt.FullName 'aapt2.exe'
    $zipalignCandidate = Join-Path $bt.FullName 'zipalign.exe'
    $apksignerCandidate = Join-Path $bt.FullName 'apksigner.bat'
    if (-not $aapt -and (Test-Path -LiteralPath $aaptCandidate)) { $aapt = $aaptCandidate }
    if (-not $aapt2 -and (Test-Path -LiteralPath $aapt2Candidate)) { $aapt2 = $aapt2Candidate }
    if (-not $zipalign -and (Test-Path -LiteralPath $zipalignCandidate)) { $zipalign = $zipalignCandidate }
    if (-not $apksigner -and (Test-Path -LiteralPath $apksignerCandidate)) { $apksigner = $apksignerCandidate }
}
$badgingTool = if ($aapt2) { $aapt2 } else { $aapt }
if ($badgingTool) {
    Write-Host ''
    Write-Host "[verify] $([IO.Path]::GetFileName($badgingTool)) badging"
    $badging = & $badgingTool dump badging $outApk
    if ($LASTEXITCODE -ne 0) { throw "$badgingTool failed on $outApk" }
    $badging | Select-String 'package:|application-label:|native-code:'
    if (($badging -join "`n") -notmatch ("package: name='" + [regex]::Escape($ExpectedPackage) + "'")) {
        throw "APK package is not $ExpectedPackage"
    }
    $nativeLine = ($badging | Select-String 'native-code:').Line
    foreach ($abiName in $AbiList) {
        if ($nativeLine -notmatch ("'" + [regex]::Escape($abiName) + "'")) {
            throw "APK does not contain requested ABI: $abiName"
        }
    }
}
if ($zipalign) {
    Write-Host '[verify] zipalign -c -v 4'
    & $zipalign -c -v 4 $outApk | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "zipalign failed on $outApk" }
}
if ($apksigner) {
    Write-Host '[verify] apksigner verify --print-certs'
    & $apksigner verify --print-certs $outApk | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "apksigner failed on $outApk" }
}

Write-Host ''
Write-Host "APK: $outApk"
Write-Host ('Size: {0:N0} bytes' -f (Get-Item -LiteralPath $outApk).Length)
$Artifacts | Format-Table -AutoSize | Out-String | Write-Host

if ($Install) {
    $adb = Resolve-FirstPath 'adb' @(
        'C:\ADB\adb.exe',
        (Join-Path $AndroidSdk 'platform-tools\adb.exe')
    )
    Invoke-Native $adb @('install', '-r', $outApk) 'adb install'
    if ($Run) {
        Invoke-Native $adb @('shell', 'am', 'start', '-n', "$ExpectedPackage/com.example.pb64app.MainActivity") 'adb start'
    }
} elseif ($Run) {
    throw '-Run requires -Install'
}

Write-Host ''
Write-Host 'Done.'
