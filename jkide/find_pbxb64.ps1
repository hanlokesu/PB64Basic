<#
  find_pbxb64.ps1 - locate the PBXB64 editor (PBXB64.exe).

  Why this exists
  ---------------
  setup.bat used to know only a few fixed locations for PBXB64.  An install
  living under the user profile - for example
  C:\Users\<name>\...\PBXB64\V6\PBX-Editor\PBXB64\PBXB64.exe - was reported as
  "PBXB64 was not found", so JKIDE.ini ended up with empty Pbxb64Exe/Pbxb64Cwd
  keys and the Compilers > PBXB64 menu stopped working.

  Output: one PBXB64.exe path per line, best candidate first. Exit 0 when at
  least one was found, exit 1 otherwise.

  Ranking: highest version folder (V6 before V5B before V5A), then newest
  PBXB64.exe, then path.

  Search order
  ------------
    1. the standard install folders (Program Files, Program Files (x86),
       C:\PBWin10) and the repository tree next to setup.bat
    2. every folder under %USERPROFILE% (C:\Users\<name>), depth 8, pruning
       AppData / node_modules / .git and similar heavy or irrelevant trees
#>
param(
    [string]$RepoRoot = '',
    [string]$UserProfile = $env:USERPROFILE,
    [int]$MaxDepth = 8
)

$ErrorActionPreference = 'SilentlyContinue'

$script:seen = @{}
$script:cands = New-Object System.Collections.ArrayList

function Add-Candidate {
    param([string]$Dir)

    if ([string]::IsNullOrWhiteSpace($Dir)) { return }

    $dir = $Dir.Trim().Trim('"').TrimEnd('\')
    if ([string]::IsNullOrWhiteSpace($dir)) { return }
    if (-not (Test-Path -LiteralPath $dir -PathType Container)) { return }

    $exe = Join-Path $dir 'PBXB64.exe'
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { return }

    $key = $dir.ToLowerInvariant()
    if ($script:seen.ContainsKey($key)) { return }
    $script:seen[$key] = $true

    # V6 / V5B / V5A in the path decide which build this is.
    $vnum = 0
    $vlet = 0
    $m = [regex]::Match($dir, '(?i)\\(?:V(\d+)([A-Z]?))\\')
    if ($m.Success) {
        $vnum = [int]$m.Groups[1].Value
        $letter = $m.Groups[2].Value
        if ($letter) { $vlet = [int][char]$letter.ToUpperInvariant()[0] }
    }

    $null = $script:cands.Add([pscustomobject]@{
        Path  = $dir
        VNum  = $vnum
        VLet  = $vlet
        MTime = (Get-Item -LiteralPath $exe).LastWriteTime
    })
}

# 1. standard install folders
if ($env:ProgramFiles) { Add-Candidate (Join-Path $env:ProgramFiles 'PBXB64\PBX-Editor\PBXB64') }
if (${env:ProgramFiles(x86)}) { Add-Candidate (Join-Path ${env:ProgramFiles(x86)} 'PBXB64\PBX-Editor\PBXB64') }
Add-Candidate 'C:\PBWin10\PBXB64\PBX-Editor\PBXB64'

# 2. the repository tree next to setup.bat (kept from the old cmd checks)
if ($RepoRoot) {
    foreach ($rel in @('PBXB64', '..\PBXB64', '..\..\PBXB64')) {
        $base = Join-Path $RepoRoot $rel
        if (-not (Test-Path -LiteralPath $base -PathType Container)) { continue }
        foreach ($v in Get-ChildItem -LiteralPath $base -Directory -Filter 'V*' -ErrorAction SilentlyContinue) {
            Add-Candidate (Join-Path $v.FullName 'PBX-Editor\PBXB64')
        }
        Add-Candidate (Join-Path $base 'PBX-Editor\PBXB64')
        Add-Candidate $base
    }
}

# 3. the user profile: this is where a personal install usually lives.
#    Skipped when the cheap checks above already found an install.  The
#    -Filter lets the filesystem provider discard non-matching directories by
#    name, which is several times faster than walking every directory in
#    managed code (about 4 s for a full profile here, 15 s for a naive walk).
if ($script:cands.Count -eq 0 -and $UserProfile -and (Test-Path -LiteralPath $UserProfile -PathType Container)) {
    $skip = @('AppData', 'node_modules', '.git', '.svn', '.hg', 'Windows', 'ProgramData',
              '$Recycle.Bin', 'System Volume Information', 'Temp', '__pycache__',
              '.cache', '.cargo', '.rustup', 'target')
    Add-Candidate (Join-Path $UserProfile 'PBXB64')
    foreach ($top in Get-ChildItem -LiteralPath $UserProfile -Directory -ErrorAction SilentlyContinue) {
        if ($skip -contains $top.Name) { continue }
        if ($top.Attributes -band [IO.FileAttributes]::ReparsePoint) { continue }
        $hits = Get-ChildItem -LiteralPath $top.FullName -Directory -Recurse -Depth $MaxDepth -Filter 'PBXB64' -ErrorAction SilentlyContinue
        foreach ($hit in $hits) { Add-Candidate $hit.FullName }
    }
}

if ($script:cands.Count -eq 0) { exit 1 }

$ranked = @($script:cands | Sort-Object `
    @{ Expression = { $_.VNum }; Descending = $true }, `
    @{ Expression = { $_.VLet }; Descending = $true }, `
    @{ Expression = { $_.MTime }; Descending = $true }, `
    @{ Expression = { $_.Path.ToLowerInvariant() } })

foreach ($c in $ranked) { Write-Output (Join-Path $c.Path 'PBXB64.exe') }

exit 0
