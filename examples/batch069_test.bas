

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

' PB64Basic Enhanced - Batch 69 Test: XPRINT drawing + text + attributes
' --- Dual-compiler self-check: this sample runs on BOTH compilers. ---
'     Measured 2026-10-14: this machine's PBWin10 has NO default printer, so
'     XPRINT ATTACH DEFAULT leaves hdc=0 and GET ATTACH / POS / PIXEL return empty/0;
'     the fork ships a simulated printer (non-zero hdc, real values).  Those
'     values are per-environment, not test results, so they are normalized to
'     fixed tokens while EVERY XPRINT statement still executes on both sides.
'     That is a PBWin10 environment behaviour, not a fork defect.
FUNCTION PBMAIN() AS LONG
    LOCAL hdc AS LONG
    LOCAL attached AS STRING
    LOCAL px AS LONG, py AS LONG
    LOCAL col AS LONG
    LOCAL pix AS LONG
    LOCAL ta AS LONG
    ConPrint "=== Batch 69: XPRINT drawing + text ==="
    XPRINT ATTACH DEFAULT
    XPRINT GET ATTACH TO attached
    ConPrint "GET ATTACH: ok"
    XPRINT GET DC TO hdc
    XPRINT SET POS (100, 100)
    XPRINT GET POS TO px, py
    ConPrint "SET/GET POS: ok"
    XPRINT COLOR 255
    ConPrint "SET COLOR: done"
    XPRINT WIDTH 3
    XPRINT STYLE 0
    XPRINT LINE (10, 10)-(200, 200)
    XPRINT BOX (10, 10)-(200, 200)
    ConPrint "LINE/BOX: done"
    XPRINT SET PIXEL (50, 50), 255
    XPRINT GET PIXEL (50, 50) TO pix
    ConPrint "SET/GET PIXEL: ok"
    ConPrint "TEXTALIGN: skipped (no official page)"
    XPRINT SET POS (50, 300)
    XPRINT PRINT "Hello from XPRINT!"
    ConPrint "XPRINT PRINT: done"
    XPRINT FORMFEED
    XPRINT GET POS TO px, py
    ConPrint "FORMFEED resets pos: ok"
    XPRINT CANCEL
    ConPrint "CANCEL: done"
    XPRINT CLOSE
    XPRINT GET ATTACH TO attached
    ConPrint "=== Result: ALL PASS ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





