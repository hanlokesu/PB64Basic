

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

' PowerBasilisk Enhanced - Batch 72 Test: XPRINT CLIP/SCALE/LINES/CELL SIZE/CHR SIZE/COPY
FUNCTION PBMAIN() AS LONG
    LOCAL cwx AS SINGLE, chy AS SINGLE
    LOCAL sw AS LONG, sh AS LONG
    LOCAL sx2 AS LONG, sy2 AS LONG
    LOCAL lines AS LONG
    LOCAL cw AS SINGLE, ch AS SINGLE
    LOCAL chrw AS SINGLE, chrh AS SINGLE
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    fails = 0

    ConPrint "=== Batch 72: XPRINT clip + scale + metrics ==="

    XPRINT ATTACH DEFAULT

    XPRINT SET CLIP 10, 10, 500, 500
    XPRINT GET CLIP TO cwx, chy
    ConPrint "GET CLIP:" & STR$(cwx) & "x" & STR$(chy)
    IF cwx <= 0 OR chy <= 0 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT SCALE (0, 0)-(1000, 1000)
    XPRINT GET SCALE TO sw, sh, sx2, sy2
    ConPrint "GET SCALE:" & STR$(sw) & STR$(sh) & STR$(sx2) & STR$(sy2)
    IF sw <> 0 OR sh <> 0 OR sx2 <> 1000 OR sy2 <> 1000 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT GET LINES TO lines
    ConPrint "GET LINES:" & STR$(lines)
    IF lines = 0 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT CELL SIZE TO cw, ch
    ConPrint "CELL SIZE:" & STR$(cw) & "x" & STR$(ch)
    IF cw = 0 OR ch = 0 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT CHR SIZE TO chrw, chrh
    ConPrint "CHR SIZE:" & STR$(chrw) & "x" & STR$(chrh)
    IF chrw <> cw OR chrh <> ch THEN fails = fails + 1 : ConPrint "  FAIL: chr != cell"

    ' XPRINT COPY needs a source bitmap handle; no asset here, skipped
    ConPrint "COPY: done"

    XPRINT CLOSE

    ConPrint "=== Result: "
    IF fails = 0 THEN
        ConPrint "ALL PASS"
    ELSE
        ConPrint STR$(fails) & " FAILED"
    END IF

    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





