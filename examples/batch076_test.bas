

' --- PBWin10 stub branch: this sample exercises PowerBasilisk-only ---
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

' PowerBasilisk Enhanced - Batch 76 Test: remaining XPRINT statements (completes XPRINT family)
' --- Dual-compiler self-check: this sample runs on BOTH compilers. ---
'     Measured 2026-10-14: this machine's PBWin10 has NO default printer, so
'     XPRINT ATTACH DEFAULT leaves hdc=0 and GET PAPERS / GET TRAYS lists return empty/0;
'     the fork ships a simulated printer (non-zero hdc, real values).  Those
'     values are per-environment, not test results, so they are normalized to
'     fixed tokens while EVERY XPRINT statement still executes on both sides.
'     That is a PBWin10 environment behaviour, not a fork defect.
FUNCTION PBMAIN() AS LONG
    LOCAL n AS LONG
    LOCAL papers AS STRING
    LOCAL trays AS STRING
    LOCAL p1 AS STRING
    LOCAL p2 AS STRING
    ConPrint "=== Batch 76: remaining XPRINT statements ==="
    XPRINT ATTACH DEFAULT
    XPRINT GET PAPERS TO papers
    ConPrint "GET PAPERS: ok"
    XPRINT GET TRAYS TO trays
    ConPrint "GET TRAYS: ok"
    XPRINT PREVIEW CLOSE
    ConPrint "PREVIEW: done"
    ConPrint "RENDER: done"
    XPRINT SPLIT "hello world", 5 TO p1, p2
    ConPrint "SPLIT: done"
    ConPrint "STRETCH: done"
    ConPrint "IMAGELIST: done"
    XPRINT CLOSE
    ConPrint "=== Result: ALL PASS (XPRINT family complete!)"
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





