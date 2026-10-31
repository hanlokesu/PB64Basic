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

OPTION EXPLICIT

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL a() AS LONG
    LOCAL sa() AS STRING
    LOCAL i AS LONG
    LOCAL ok AS LONG
    ok = 0

    ' --- enlarge: REDIM PRESERVE 1 TO 3 then 1 TO 5 (CHW: plain REDIM is
    '     ERASE+DIM and zeroes; PRESERVE keeps old element values) ---
    REDIM a(1 TO 3)
    a(1) = 1: a(2) = 2: a(3) = 3
    REDIM PRESERVE a(1 TO 5)
    a(4) = 4: a(5) = 5
    FOR i = 1 TO 5
        ConPrint "a(" & STR$(i) & ")=" & STR$(a(i))
        IF a(i) <> i THEN ok = ok + 1
    NEXT i

    ' --- shrink: REDIM 1 TO 5 then 1 TO 2 ---
    REDIM a(1 TO 2)
    a(1) = 7: a(2) = 8
    IF a(1) <> 7 OR a(2) <> 8 THEN ok = ok + 10

    ' --- string array enlarged (PRESERVE keeps sa(1..2)) ---
    REDIM sa(1 TO 2)
    sa(1) = "a": sa(2) = "b"
    REDIM PRESERVE sa(1 TO 4)
    sa(3) = "c": sa(4) = "d"
    IF sa(1) <> "a" OR sa(4) <> "d" THEN ok = ok + 100

    ' --- REDIM inside IF block (plain REDIM zeroes; only a(6) is checked) ---
    IF 1 = 1 THEN
        REDIM a(1 TO 6)
        a(6) = 99
    END IF
    IF a(6) <> 99 THEN ok = ok + 1000

    IF ok = 0 THEN
        ConPrint "REDIM ALL PASS"
        FUNCTION = 0
    ELSE
        ConPrint "REDIM FAIL code=" & STR$(ok)
        FUNCTION = 1
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


