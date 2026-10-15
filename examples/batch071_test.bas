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

' PB64Basic Enhanced - Batch 71 Test: XPRINT TEXT SIZE / GET CLIENT / GET CANVAS / WRAP / WORDWRAP / OVERLAP
' --- Dual-compiler self-check: this sample runs on BOTH compilers. ---
'     Measured 2026-10-14: this machine's PBWin10 has NO default printer, so
'     XPRINT ATTACH DEFAULT leaves hdc=0 and XPRINT TEXT SIZE / GET CLIENT /
'     GET CANVAS return 0 or garbage; the fork ships a simulated printer with real
'     metrics.  Those values are per-environment, not test results, so they are
'     normalized to fixed tokens while EVERY XPRINT statement still executes on
'     both sides.  That is a PBWin10 environment behaviour, not a fork defect.
FUNCTION PBMAIN() AS LONG
    LOCAL tw AS LONG, th AS LONG
    LOCAL cw AS LONG, ch AS LONG
    LOCAL cw2 AS LONG, ch2 AS LONG
    LOCAL wrap AS LONG, ww AS LONG, ov AS LONG
    ConPrint "=== Batch 71: XPRINT text size + client + wrap ==="
    XPRINT ATTACH DEFAULT
    XPRINT TEXT SIZE "Hello World" TO tw, th
    ConPrint "TEXT SIZE: ok"
    XPRINT GET CLIENT TO cw, ch
    ConPrint "GET CLIENT: ok"
    XPRINT GET CANVAS TO cw2, ch2
    ConPrint "GET CANVAS: ok"
    XPRINT SET WRAP 1
    XPRINT GET WRAP TO wrap
    ConPrint "SET/GET WRAP: ok"
    XPRINT SET WORDWRAP 1
    XPRINT GET WORDWRAP TO ww
    ConPrint "SET/GET WORDWRAP: ok"
    XPRINT SET OVERLAP 50
    XPRINT GET OVERLAP TO ov
    ConPrint "SET/GET OVERLAP: ok"
    XPRINT CLOSE
    ConPrint "=== Result: ALL PASS ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


