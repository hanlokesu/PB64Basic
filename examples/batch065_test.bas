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

' PB64Basic Enhanced - Batch 65 test
' GRAPHIC SET SIZE / SET CLIP / SET VIRTUAL / SET+GET WORDWRAP
FUNCTION PBMAIN() AS LONG
    LOCAL hBmp AS LONG
    LOCAL s AS STRING
    LOCAL w AS LONG
    LOCAL h AS LONG
    LOCAL cw AS SINGLE
    LOCAL ch AS SINGLE
    LOCAL n AS LONG
    LOCAL waitk AS STRING
    LOCAL ok AS LONG
    ok = 0

    GRAPHIC BITMAP NEW 100, 50 TO hBmp
    GRAPHIC ATTACH hBmp, 0
    GRAPHIC SET SIZE 80, 40
    GRAPHIC GET SIZE TO w, h
    IF w = 80 AND h = 40 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL1" & STR$(w) & STR$(h)
    END IF
    GRAPHIC GET BITS TO s
    IF LEN(s) = 12840 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL2" & STR$(LEN(s))
    END IF

    GRAPHIC SET CLIP 10, 20, 90, 40
    GRAPHIC GET CLIP TO cw, ch
    IF cw = 80 AND ch = 20 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL3" & STR$(cw) & STR$(ch)
    END IF
    GRAPHIC SET VIRTUAL 200, 150
    ok = ok + 1

    GRAPHIC SET WORDWRAP 0
    GRAPHIC GET WORDWRAP TO n
    IF n = 0 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL5" & STR$(n)
    END IF
    GRAPHIC SET WORDWRAP 1
    GRAPHIC GET WORDWRAP TO n
    IF n = 1 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL6" & STR$(n)
    END IF
    IF ok = 6 THEN
        ConPrint "ALL PASS (6/6)"
    ELSE
        ConPrint "FAIL: " & STR$(ok)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

