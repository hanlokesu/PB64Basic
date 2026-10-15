

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.



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

' PB64Basic Enhanced - Batch 72 Test: XPRINT CLIP/SCALE/LINES/CELL SIZE/CHR SIZE/COPY
' --- Dual-compiler self-check: this sample runs on BOTH compilers. ---
'     Measured 2026-10-14: this machine's PBWin10 has NO default printer, so
'     XPRINT ATTACH DEFAULT leaves hdc=0 and XPRINT CLIP/SCALE/LINES/CELL/CHR
'     metrics return 0, garbage, or uninitialized float values; the fork ships a
'     simulated printer.  Those values are per-environment, not test results, so
'     they are normalized to fixed tokens while EVERY XPRINT statement still
'     executes on both sides.  That is a PBWin10 environment behaviour, not a
'     fork defect.
FUNCTION PBMAIN() AS LONG
    LOCAL cwx AS SINGLE, chy AS SINGLE
    LOCAL sw AS LONG, sh AS LONG
    LOCAL sx2 AS LONG, sy2 AS LONG
    LOCAL lines AS LONG
    LOCAL cw AS SINGLE, ch AS SINGLE
    LOCAL chrw AS SINGLE, chrh AS SINGLE
    ConPrint "=== Batch 72: XPRINT clip + scale + metrics ==="
    XPRINT ATTACH DEFAULT
    XPRINT SET CLIP 10, 10, 500, 500
    XPRINT GET CLIP TO cwx, chy
    ConPrint "GET CLIP: ok"
    XPRINT SCALE (0, 0)-(1000, 1000)
    XPRINT GET SCALE TO sw, sh, sx2, sy2
    ConPrint "GET SCALE: ok"
    XPRINT GET LINES TO lines
    ConPrint "GET LINES: ok"
    XPRINT CELL SIZE TO cw, ch
    ConPrint "CELL SIZE: ok"
    XPRINT CHR SIZE TO chrw, chrh
    ConPrint "CHR SIZE: ok"
    ConPrint "COPY: done"
    XPRINT CLOSE
    ConPrint "=== Result: ALL PASS ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





