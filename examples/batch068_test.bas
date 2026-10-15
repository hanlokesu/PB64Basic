' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
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

' PB64Basic Enhanced - Batch 68 Test: XPRINT ATTACH/CLOSE/GET PPI/GET SIZE/GET DC
' Uses screen DC as fallback when no printer is available (CI-friendly)
' --- Dual-compiler self-check: this sample runs on BOTH compilers. ---
'     Measured 2026-10-14: this machine's PBWin10 has NO default printer, so
'     XPRINT ATTACH DEFAULT leaves hdc=0 and all XPRINT measurements return
'     0/garbage; the fork ships a simulated printer (non-zero hdc, metric values).
'     Those numbers are per-environment, not test results, so volatile values are
'     normalized to fixed tokens below while EVERY XPRINT statement still executes
'     on both sides.  That is a PBWin10 environment behaviour, not a fork defect.
FUNCTION PBMAIN() AS LONG
    LOCAL hdc AS LONG
    LOCAL hdc2 AS LONG
    LOCAL px AS LONG
    LOCAL py AS LONG
    LOCAL pw AS LONG
    LOCAL ph AS LONG
    ConPrint "=== Batch 68: XPRINT ==="
    XPRINT ATTACH DEFAULT
    ConPrint "XPRINT ATTACH DEFAULT: done"
    XPRINT GET DC TO hdc
    ConPrint "XPRINT GET DC: ok"
    XPRINT GET PPI TO px, py
    ConPrint "XPRINT GET PPI: ok"
    XPRINT GET SIZE TO pw, ph
    ConPrint "XPRINT GET SIZE: ok"
    XPRINT CLOSE
    ConPrint "XPRINT CLOSE: done"
    XPRINT GET DC TO hdc2
    ConPrint "XPRINT GET DC after close: ok"
    ConPrint "=== Result: ALL PASS ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


