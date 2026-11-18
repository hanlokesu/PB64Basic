# PB64Basic Enhanced - JK-IDE help installer (batch 380)
# Copies the bundled PB64Basic CHM/CHW into JK-IDE, keeps a one-time backup of
# the original PBWin help, and points both help paths at the bundled files.
param(
    [Parameter(Mandatory=$true)][string]$JkIdeDir,
    [Parameter(Mandatory=$true)][string]$SourceDir
)
$ErrorActionPreference = 'Stop'

$JkIdeDir = (Resolve-Path -LiteralPath $JkIdeDir).Path.TrimEnd('\')
$SourceDir = (Resolve-Path -LiteralPath $SourceDir).Path.TrimEnd('\')

function Assert-NotLocked([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { return }
    try {
        $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
        $stream.Close()
    } catch {
        throw "JK-IDE is using $Path. Close JK-IDE completely, then run setup again."
    }
}

foreach ($name in @('PB.chm', 'PB.chw', 'PB64Basic.chm', 'PB64Basic.chw')) {
    Assert-NotLocked (Join-Path $JkIdeDir $name)
}
$srcChm = Join-Path $SourceDir 'PB64Basic.chm'
$srcChw = Join-Path $SourceDir 'PB64Basic.chw'
foreach ($p in @($srcChm, $srcChw)) {
    if (-not (Test-Path -LiteralPath $p)) { throw "missing bundled help file: $p" }
}

function Backup-Once([string]$Path, [string]$Backup) {
    if ((Test-Path -LiteralPath $Path) -and -not (Test-Path -LiteralPath $Backup)) {
        Copy-Item -LiteralPath $Path -Destination $Backup -Force
        Write-Host "  [backup] $Path -> $Backup"
    }
}

# Keep the pre-integration PBWin help for rollback.  The backup is created once;
# re-running setup must not replace it with the already-installed PB64Basic copy.
Backup-Once (Join-Path $JkIdeDir 'PB.chm') (Join-Path $JkIdeDir 'PB.chm.pbwin-bak')
Backup-Once (Join-Path $JkIdeDir 'PB.chw') (Join-Path $JkIdeDir 'PB.chw.pbwin-bak')

# PB.chm / PB.chw are the names JK-IDE expects.  PB64Basic.chm / .chw are kept
# beside them so a user can open the reference by its real name.
Copy-Item -LiteralPath $srcChm -Destination (Join-Path $JkIdeDir 'PB.chm') -Force
Copy-Item -LiteralPath $srcChw -Destination (Join-Path $JkIdeDir 'PB.chw') -Force
Copy-Item -LiteralPath $srcChm -Destination (Join-Path $JkIdeDir 'PB64Basic.chm') -Force
Copy-Item -LiteralPath $srcChw -Destination (Join-Path $JkIdeDir 'PB64Basic.chw') -Force

function Set-IniValue([string]$Path, [string]$Section, [string]$Key, [string]$Value) {
    if (-not (Test-Path -LiteralPath $Path)) { throw "missing ini file: $Path" }
    $text = [IO.File]::ReadAllText($Path)
    $nl = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
    $pattern = "(?m)^[ \t]*" + [regex]::Escape($Key) + "[ \t]*=.*$"
    $line = "$Key=$Value"
    $rx = [regex]::new($pattern)
    if ($rx.IsMatch($text)) {
        $text = $rx.Replace($text, $line, 1)
    } else {
        $sectionPattern = "(?m)^[ \t]*\[" + [regex]::Escape($Section) + "\][ \t]*$"
        $srx = [regex]::new($sectionPattern)
        $m = $srx.Match($text)
        if (-not $m.Success) { throw "section [$Section] not found in $Path" }
        $idx = $m.Index + $m.Length
        $text = $text.Substring(0, $idx) + $nl + $line + $text.Substring($idx)
    }
    [IO.File]::WriteAllText($Path, $text, (New-Object System.Text.UTF8Encoding($false)))
}

Set-IniValue (Join-Path $JkIdeDir 'JKIDE.ini') 'Compilers' 'PbHelpFile' (Join-Path $JkIdeDir 'PB64Basic.chm')
Set-IniValue (Join-Path $JkIdeDir 'JK-IDE.ini') 'Compiler options' 'PBWinHelpPath' (Join-Path $JkIdeDir 'PB.chm')

Write-Host "[OK] installed PB64Basic help in $JkIdeDir"
Write-Host "     PB.chm / PB.chw             = native F1 target"
Write-Host "     PB64Basic.chm / .chw        = explicit reference"
Write-Host "     original PBWin help backup  = *.pbwin-bak"
