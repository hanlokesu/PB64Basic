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

' PB64Basic Enhanced - Batch 64 test
' GRAPHIC GET BITS / SET BITS / GET SCALE / SCALE / SET AUTOSIZE
FUNCTION PBMAIN() AS LONG
    LOCAL hBmp AS LONG
    LOCAL hBmp2 AS LONG
    LOCAL s AS STRING
    LOCAL w AS LONG
    LOCAL h AS LONG
    LOCAL x1 AS SINGLE
    LOCAL y1 AS SINGLE
    LOCAL x2 AS SINGLE
    LOCAL y2 AS SINGLE
    LOCAL waitk AS STRING
    LOCAL ok AS LONG
    ok = 0

    GRAPHIC BITMAP NEW 100, 50 TO hBmp
    GRAPHIC ATTACH hBmp, 0
    GRAPHIC GET SIZE TO w, h
    IF w = 100 AND h = 50 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL1"
    END IF

    GRAPHIC GET BITS TO s
    IF LEN(s) = 20040 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL2 len=" & STR$(LEN(s))
    END IF

    GRAPHIC BITMAP NEW 50, 25 TO hBmp2
    GRAPHIC ATTACH hBmp2, 0
    GRAPHIC GET SIZE TO w, h
    IF w = 50 AND h = 25 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL3"
    END IF

    GRAPHIC SET BITS s
    GRAPHIC GET SIZE TO w, h
    IF w = 100 AND h = 50 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL4"
    END IF

    GRAPHIC GET SCALE TO x1, y1, x2, y2
    IF x1 = 0 AND y1 = 0 AND x2 = 100 AND y2 = 50 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL5" & STR$(x1) & STR$(y1) & STR$(x2) & STR$(y2)
    END IF
    GRAPHIC SCALE (10, 20) - (110, 120)
    GRAPHIC GET SCALE TO x1, y1, x2, y2
    IF x1 = 10 AND y1 = 20 AND x2 = 110 AND y2 = 120 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL6" & STR$(x1) & STR$(y1) & STR$(x2) & STR$(y2)
    END IF
    GRAPHIC SCALE PIXELS
    GRAPHIC GET SCALE TO x1, y1, x2, y2
    IF x1 = 0 AND y1 = 0 AND x2 = 100 AND y2 = 50 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL7" & STR$(x1) & STR$(y1) & STR$(x2) & STR$(y2)
    END IF
    GRAPHIC SET AUTOSIZE 200, 150
    ok = ok + 1

    IF ok = 8 THEN
        ConPrint "ALL PASS (8/8)"
    ELSE
        ConPrint "FAIL: " & STR$(ok)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

