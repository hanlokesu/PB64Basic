# JK-IDE integration for PB64Basic

PB64Basic Enhanced ships with a ready-made JK-IDE addon that adds a
**Compilers** menu to the JK-IDE menu bar. From that menu you can compile
the active file with **PBWin 10**, **PB64Basic** or **PBXB64** without
leaving the IDE.

## What is in this folder

| file | purpose |
|---|---|
| `Setup.bat` (zip root) | one-shot installer: finds JK-IDE (auto-downloads it if missing), copies the addon, writes `JKIDE.ini`, enables the addon |
| `User_Setup.bas` | addon source - installs the Compilers menu at IDE startup |
| `User_Code.bas` | addon source - reads `JKIDE.ini` and runs the requested compiler |
| `User_Command.bas`, `User_Menu.bas`, `Userdll.bas` | addon build files (do not edit) |
| `userdll.dll` | pre-built addon (PBWin 10 compiled, 73216 bytes) |
| `JKIDE.ini` | configuration template - all compiler paths live here |
| `enable_userdll.ps1` | helper used by Setup.bat to set `Userdll=1` in the IDE's own `JK-IDE.ini` |

## How to install

1. Unpack the PB64Basic zip anywhere on the target machine.
2. Double-click `Setup.bat` (in the zip root).
3. Answer `Y` to the "JK-IDE integration" question.

Then one of two things happens:

* **JK-IDE already installed** (typical `C:\PBWin10\JK-IDE` or
  `%ProgramFiles(x86)%\JK-IDE`): it is detected automatically and the addon
  is installed into it.
* **JK-IDE not found**: the setup asks *"Auto-download it from the author
  official site (~43 MB)? (Y/N)"*.
  * `Y` - downloads `jkb.zip` from the author's official Dropbox, asks where
    to unpack it (Enter = `C:\JK-IDE`), extracts and installs the addon.
  * `N` - you are asked for the folder that contains `jk-ide.exe`
    (Enter alone = skip the integration).

4. Restart JK-IDE.

The `Compilers` menu appears between `Options` and `Tools`. Its items:

* **PBWin 10** - PowerBASIC for Windows 10 (`/IC:\PBWin10\WINAPI /L /Q`)
* **PB64Basic** - this project's 64-bit compiler
* **PBXB64** - PowerBASIC 64-bit fork (optional, detected if installed)
* **Compile (default)** - JK-IDE's own default compile action

### Updating an existing JK-IDE install (important)

`Setup.bat` **always overwrites the addon files** whenever you answer `Y` to
the JK-IDE question - the five `User_*.bas` sources go into the `Addon\`
folder and the prebuilt `userdll.dll` replaces the one next to `jk-ide.exe`.
So if you already have JK-IDE (installed by an earlier PB64Basic release, by
the author's package, or manually):

* **Re-run `Setup.bat` from the new zip and answer `Y`** - it detects the
  existing JK-IDE and copies the new addon over the old one. This is the
  supported upgrade path; do not copy individual files by hand.
* Close JK-IDE before re-running `Setup.bat` (the running IDE locks
  `userdll.dll` and the copy would fail), then start it again. The menu and
  all fixes from the new version are then active.

### Compile-error reporting

The menu reports compile errors the way JK-IDE itself does:

* **PBWin 10** runs JK-IDE's own compile command (the same as F9): the error
  line is highlighted in the editor, marked with a red dot in the gutter,
  annotated with a tooltip, and the caret jumps to it.
* **PB64Basic / PBXB64** are external compilers the IDE does not know, so the
  addon fakes the native behaviour: the compile log is streamed to the
  Output Window (lines containing "Error " turn red automatically) and the
  editor jumps to the first error line with a red marker.
* Success shows the compiler output and `EXITCODE=0` in the Output Window.

## If the auto-download fails

The download URL is the author's own Dropbox link (Juergen Kuehlwein, thread
"Download and setup" at http://pump.richheimer.de/showthread.php?tid=120).
If the link ever changes or the download is blocked (a proxy, firewall or
corporate policy), open that page in a browser, download `jkb.zip` manually,
unpack it anywhere, then re-run `Setup.bat` and enter that folder when
asked.

Notes about the package:

* It is portable ("path independent"): no registry writes, no install step.
* It is **not digitally signed**; Windows SmartScreen and some antivirus
  tools may show a warning - this is normal for this project, allow it.
* The first launch of JK-IDE asks you to accept its license; afterwards the
  `Compilers` menu is already present because the addon is enabled.

## Configuration (`JKIDE.ini`)

`JKIDE.ini` sits in the JK-IDE folder, next to `userdll.dll`. Keys:

| key | meaning |
|---|---|
| `PbwinExe` / `PbwinCwd` / `PbwinInc` | PBWin executable, its working dir, its WINAPI include dir (no trailing space; `/I` path must be space-free) |
| `Pb64Exe` / `Pb64Cwd` / `Pb64Runtime` | PB64Basic executable, working dir, runtime object name |
| `Pbxb64Exe` / `Pbxb64Cwd` | PBXB64 executable and working dir |

Leave a key empty if that compiler is not installed - the menu then shows a
clear message instead of running. Paths with spaces are allowed in the
executable and working-dir keys; the PBWin include dir must stay space-free
(PBWin itself rejects quoted `/I` paths).

## Requirements

* JK-IDE 2.4.x (free, closed source) - installed automatically by
  `Setup.bat` if missing, or get it from the PowerBASIC forums
  (http://pump.richheimer.de/showthread.php?tid=120).
* At least one of the compilers above. PBWin 10 and PBXB64 are commercial
  PowerBASIC products; PB64Basic is free and included in this zip.
* The first PBWin compile needs `C:\PBWin10\WINAPI` (the stock install
  layout). If PBWin lives elsewhere, edit `PbwinExe` / `PbwinCwd` /
  `PbwinInc` in `JKIDE.ini`.

## Rebuilding the addon yourself

Install PBWin 10, open `Addon\Userdll.bas` in JK-IDE and compile
(`#utility compile10` builds `Userdll.dll` into the Addon folder). Copy it
over the one in the JK-IDE root while JK-IDE is closed. Source files must be
kept CRLF and ASCII-only (PBWin rejects other encodings).

Note: compiling the addon needs `Visualdesigner\controls\*.inc` (e.g.
`link.inc`) inside the JK-IDE folder. The official `jkb.zip` (as of
2026-04-16) ships that folder empty, so a freshly auto-downloaded JK-IDE
cannot recompile the addon and fails with Error 493; the prebuilt
`userdll.dll` included in this zip does not need it. If you hit 493, copy
`Visualdesigner\controls` from a complete JK-IDE 2.4 install (e.g.
`C:\PBWin10\JK-IDE`).
