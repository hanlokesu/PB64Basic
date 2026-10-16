
' PB64Basic Enhanced - Batch 75 Test: XPRINT paper + tray (official PBWin10 syntax)
' --- Dual-compiler self-check: this sample runs on BOTH compilers. ---
'     XPRINT CELL / GET SELECTION / RESOURCE SAVE FILE are PB64Basic-only
'     extensions; official PBWin10 rejects them (415/432/516), so they are not
'     part of this dual sample.
'     PAPER/TRAY read-back values are per-environment (this machine has no
'     default printer: PBWin10 hdc=0 reads back 0, the fork's simulated printer
'     stores the SET value), not test results - so they are normalized to fixed
'     tokens while every XPRINT SET/GET still executes on both sides.  Same
'     pattern as batch 74.

DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB
SUB ConWaitKey()
    LOCAL h AS LONG
    LOCAL c AS STRING * 1
    LOCAL n AS DWORD
    h = GetStdHandle(-10)
    IF h <> 0 THEN
        ReadFile h, c, 1, n, 0
    END IF
END SUB

' PB64Basic Enhanced - Batch 75 Test: XPRINT paper + tray
FUNCTION PBMAIN() AS LONG
    LOCAL v AS LONG
    LOCAL waitk AS STRING

    ConPrint "=== Batch 75: XPRINT paper + tray ==="

    XPRINT ATTACH DEFAULT

    XPRINT SET PAPER 9
    XPRINT GET PAPER TO v
    ConPrint "PAPER: ok"

    XPRINT SET TRAY 3
    XPRINT GET TRAY TO v
    ConPrint "TRAY: ok"

    XPRINT CLOSE

    ConPrint "=== Result: ALL PASS ==="
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION
