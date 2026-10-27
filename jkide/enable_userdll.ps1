# enable_userdll.ps1 - set Userdll=1 in JK-IDE.ini (ANSI/GBK safe).
# JK-IDE.ini (with hyphen) is the IDE's own config; create it if absent so a
# fresh install gets Userdll=1 before the first IDE launch.
param([string]$JkIdeDir)
# defensive: a cmd caller may hand us a trailing backslash (%~dpF) or even a
# literal quote when the trailing backslash ate the closing quote in Win32
# command-line parsing; a quote inside the path makes every API call throw.
$JkIdeDir = $JkIdeDir.TrimEnd('\"')
$p = Join-Path $JkIdeDir 'JK-IDE.ini'
$enc = [System.Text.Encoding]::GetEncoding(936)
if (-not (Test-Path -LiteralPath $p)) {
    [System.IO.File]::WriteAllText($p, "[Editor options]`r`nUserdll=1`r`n", $enc)
    Write-Output 'Userdll=1 CREATED'; exit 0
}
$c = [System.IO.File]::ReadAllText($p, $enc)
if ($c -match '(?m)^Userdll=0') { $c = $c -replace '(?m)^Userdll=0', 'Userdll=1' }
elseif ($c -notmatch '(?m)^Userdll=') { $c = $c.TrimEnd([char]13,[char]10) + [Environment]::NewLine + 'Userdll=1' + [Environment]::NewLine }
[System.IO.File]::WriteAllText($p, $c, $enc)
Write-Output 'Userdll=1 OK'
