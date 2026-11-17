<#
  find_jkide.ps1 - locate every JK-IDE installation that contains jk-ide.exe.

  Why this exists
  ---------------
  setup.bat used to check exactly two hard-coded paths
  ("%ProgramFiles(x86)%\JK-IDE" and "C:\PBWin10\JK-IDE"). A stock install in
  D:\jk-ide - or on any other drive - was therefore reported as "JK-IDE not
  found" and the user was pushed into a 43 MB download that was not needed.

  Output
  ------
  One folder per line, best candidate first, without a trailing backslash.
  Exit 0 when at least one folder was found, exit 1 when none was found.

  Several installations are normal (an old copy in Program Files, the working
  copy on D:, a test copy in a tools folder).  This script does NOT silently
  pick one: it prints all of them, ranked, and setup.bat lets the user choose.
  Ranking:
    1. the folder selected on a previous run (stable default)
    2. highest jk-ide.exe file version
    3. newest jk-ide.exe timestamp
    4. a standard install folder (Program Files / Program Files (x86) / C:\PBWin10)
    5. path, for a deterministic order

  Search order, cheapest first
  ----------------------------
    1. the folder remembered from a previous run
    2. the well-known install folders
    3. App Paths and JK-IDE registry keys
    4. every local (non-cloud) drive: <drive>\JK-IDE and the same name one
       level down (D:\Tools\JK-IDE, ...), plus a direct jk-ide.exe in any
       first-level folder
    5. a bounded recursive scan (depth 3) as the last resort
#>
param(
    [string]$CacheFile = (Join-Path $env:LOCALAPPDATA 'PB64Basic\jkide_dir.txt'),
    [int]$MaxDepth = 3
)

$ErrorActionPreference = 'SilentlyContinue'

$script:seen = @{}
$script:cands = New-Object System.Collections.ArrayList

function Add-Candidate {
    param([string]$Dir, [switch]$NameMustMatch)

    if ([string]::IsNullOrWhiteSpace($Dir)) { return }

    $dir = $Dir.Trim().Trim('"').TrimEnd('\')
    if ([string]::IsNullOrWhiteSpace($dir)) { return }
    if (-not (Test-Path -LiteralPath $dir -PathType Container)) { return }

    # A drive scan must not turn "D:\JK-IDE-OLD" or "...\xxJK-IDE" into an
    # install candidate just because a jk-ide.exe happens to sit there.
    if ($NameMustMatch -and ((Split-Path -Leaf $dir) -ine 'JK-IDE')) { return }

    $exe = Join-Path $dir 'jk-ide.exe'
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { return }

    # Windows paths are case-insensitive: D:\JK-IDE and d:\jk-ide are one install.
    $key = $dir.ToLowerInvariant()
    if ($script:seen.ContainsKey($key)) { return }
    $script:seen[$key] = $true

    $item = Get-Item -LiteralPath $exe
    $ver = [version]'0.0'
    if ($item.VersionInfo.FileVersion) {
        $m = [regex]::Match([string]$item.VersionInfo.FileVersion, '\d+(\.\d+)+')
        if ($m.Success) {
            try { $ver = [version]$m.Value } catch { }
        }
    }

    $null = $script:cands.Add([pscustomobject]@{
        Path     = $dir
        Version  = $ver
        MTime    = $item.LastWriteTime
        Standard = ($dir -match '(?i)^([A-Z]:\\Program Files( \(x86\))?\\JK-IDE|C:\\PBWin10\\JK-IDE)$')
    })
}

function Save-Cache {
    param([string]$Dir)
    if ([string]::IsNullOrWhiteSpace($CacheFile)) { return }
    try {
        $parent = Split-Path -Parent $CacheFile
        if (-not (Test-Path -LiteralPath $parent)) {
            New-Item -ItemType Directory -Path $parent -Force | Out-Null
        }
        [IO.File]::WriteAllText($CacheFile, $Dir, [Text.UTF8Encoding]::new($false))
    } catch { }
}

# The folder the user picked last time.  It only changes the ranking, never
# short-circuits the scan: a second install that appeared meanwhile must still
# be visible in the list.
$cached = $null
if ($CacheFile -and (Test-Path -LiteralPath $CacheFile -PathType Leaf)) {
    $line = Get-Content -LiteralPath $CacheFile -TotalCount 1
    if ($line) {
        $cached = $line.Trim().Trim('"').TrimEnd('\')
        Add-Candidate $cached
    }
}

# 2. well-known folders
if ($env:ProgramFiles) { Add-Candidate (Join-Path $env:ProgramFiles 'JK-IDE') }
if (${env:ProgramFiles(x86)}) { Add-Candidate (Join-Path ${env:ProgramFiles(x86)} 'JK-IDE') }
Add-Candidate 'C:\PBWin10\JK-IDE'

# 3. registry: App Paths and the product keys
$keys = @(
    'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\jk-ide.exe',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\jk-ide.exe',
    'HKLM:\SOFTWARE\JK-IDE',
    'HKLM:\SOFTWARE\WOW6432Node\JK-IDE',
    'HKCU:\SOFTWARE\JK-IDE'
)
foreach ($key in $keys) {
    if (-not (Test-Path -LiteralPath $key)) { continue }
    $props = Get-ItemProperty -LiteralPath $key
    foreach ($name in @('(default)', 'Path', 'InstallDir', 'InstallPath', 'JkIdeDir', 'ExePath')) {
        $value = $props.$name
        if ([string]::IsNullOrWhiteSpace($value)) { continue }
        if (Test-Path -LiteralPath $value -PathType Container) {
            Add-Candidate $value
        } else {
            Add-Candidate (Split-Path -LiteralPath $value -Parent)
        }
    }
}

# 4. every local drive, shallow
$localFormats = @('NTFS', 'ReFS', 'FAT', 'FAT32', 'exFAT')
$drives = @([System.IO.DriveInfo]::GetDrives() |
    Where-Object { $_.IsReady -and $_.DriveType -eq 'Fixed' -and $localFormats -contains $_.DriveFormat })
$skip = @('Windows', 'ProgramData', '$Recycle.Bin', 'System Volume Information', 'Recovery', 'PerfLogs', 'node_modules')
foreach ($d in $drives) {
    $root = $d.RootDirectory.FullName
    foreach ($name in @('JK-IDE', 'jk-ide', 'JKIDE', 'jkide')) { Add-Candidate (Join-Path $root $name) -NameMustMatch }
    $children = Get-ChildItem -LiteralPath $root -Directory -ErrorAction SilentlyContinue |
        Where-Object { $skip -notcontains $_.Name }
    foreach ($child in $children) {
        Add-Candidate $child.FullName -NameMustMatch
        foreach ($name in @('JK-IDE', 'jk-ide')) { Add-Candidate (Join-Path $child.FullName $name) -NameMustMatch }
    }
}

# 5. Last resort only: a bounded recursive scan.  Walking every drive to depth
# 3 costs a minute on a machine with several drives, so it must not run when
# the shallow pass already found an install.
if ($script:cands.Count -eq 0) {
    foreach ($d in $drives) {
        $hits = Get-ChildItem -LiteralPath $d.RootDirectory.FullName -Directory -Recurse -Depth $MaxDepth -Filter 'JK-IDE' -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -notmatch '\\(Windows|ProgramData|node_modules|System Volume Information|Recycle\.Bin)\\' }
        foreach ($hit in $hits) { Add-Candidate $hit.FullName -NameMustMatch }
    }
}

if ($script:cands.Count -eq 0) { exit 1 }

$ranked = @($script:cands | Sort-Object `
    @{ Expression = { if ($cached -and $_.Path -ieq $cached) { 0 } else { 1 } } }, `
    @{ Expression = { $_.Version }; Descending = $true }, `
    @{ Expression = { $_.MTime }; Descending = $true }, `
    @{ Expression = { if ($_.Standard) { 0 } else { 1 } } }, `
    @{ Expression = { $_.Path.ToLowerInvariant() } })

foreach ($c in $ranked) { Write-Output $c.Path }

# With one install the answer is unambiguous and can be remembered here.  With
# several, setup.bat asks first and stores the selected folder afterwards.
if ($ranked.Count -eq 1) { Save-Cache $ranked[0].Path }

exit 0
