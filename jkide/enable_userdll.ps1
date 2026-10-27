# enable_userdll.ps1 - set Userdll=1 in JK-IDE.ini (ANSI/GBK safe).
param([string]$JkIdeDir)
$p = Join-Path $JkIdeDir 'JK-IDE.ini'
if (-not (Test-Path -LiteralPath $p)) { Write-Output 'NO_JKIDE_INI'; exit 0 }
$enc = [System.Text.Encoding]::GetEncoding(936)
$c = [System.IO.File]::ReadAllText($p, $enc)
if ($c -match '(?m)^Userdll=0') { $c = $c -replace '(?m)^Userdll=0', 'Userdll=1' }
elseif ($c -notmatch '(?m)^Userdll=') { $c = $c.TrimEnd([char]13,[char]10) + [Environment]::NewLine + 'Userdll=1' + [Environment]::NewLine }
[System.IO.File]::WriteAllText($p, $c, $enc)
Write-Output 'Userdll=1 OK'
