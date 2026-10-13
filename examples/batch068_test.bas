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

' PowerBasilisk Enhanced - Batch 68 Test: XPRINT ATTACH/CLOSE/GET PPI/GET SIZE/GET DC
' Uses screen DC as fallback when no printer is available (CI-friendly)
FUNCTION PBMAIN() AS LONG
    LOCAL hdc AS LONG
    LOCAL hdc2 AS LONG
    LOCAL px AS LONG
    LOCAL py AS LONG
    LOCAL pw AS LONG
    LOCAL ph AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    fails = 0

    ConPrint "=== Batch 68: XPRINT ==="

    ' 1. ATTACH DEFAULT (falls back to screen DC if no printer)
    XPRINT ATTACH DEFAULT
    ConPrint "XPRINT ATTACH DEFAULT: done"

    ' 2. GET DC â€? should be non-zero
    XPRINT GET DC TO hdc
    ConPrint "XPRINT GET DC: hdc=" & STR$(hdc)
    IF hdc = 0 THEN
        ConPrint "  FAIL: hdc is zero"
        fails = fails + 1
    ELSE
        ConPrint "  PASS: hdc non-zero"
    END IF

    ' 3. GET PPI â€? screen is typically 96 DPI
    XPRINT GET PPI TO px, py
    ConPrint "XPRINT GET PPI: x=" & STR$(px) & " y=" & STR$(py)
    IF px = 0 OR py = 0 THEN
        ConPrint "  FAIL: PPI is zero"
        fails = fails + 1
    ELSE
        ConPrint "  PASS: PPI non-zero"
    END IF

    ' 4. GET SIZE â€? physical width/height in pixels
    XPRINT GET SIZE TO pw, ph
    ConPrint "XPRINT GET SIZE: w=" & STR$(pw) & " h=" & STR$(ph)
    IF pw = 0 OR ph = 0 THEN
        ConPrint "  FAIL: SIZE is zero"
        fails = fails + 1
    ELSE
        ConPrint "  PASS: SIZE non-zero"
    END IF

    ' 5. CLOSE
    XPRINT CLOSE
    ConPrint "XPRINT CLOSE: done"

    ' 6. GET DC after CLOSE â€? should be zero
    XPRINT GET DC TO hdc2
    ConPrint "XPRINT GET DC (after close): hdc=" & STR$(hdc2)
    IF hdc2 <> 0 THEN
        ConPrint "  FAIL: hdc should be zero after close"
        fails = fails + 1
    ELSE
        ConPrint "  PASS: hdc zero after close"
    END IF

    ConPrint "=== Result: "
    IF fails = 0 THEN
        ConPrint "ALL PASS (5/5)"
    ELSE
        ConPrint STR$(fails) & " FAILED"
    END IF

    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


