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

' Batch 7: LOF / LOC / SEEK functions
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL f AS LONG
    LOCAL len1 AS QUAD
    LOCAL pos1 AS QUAD

    f = FREEFILE
    OPEN "lof_test.tmp" FOR OUTPUT AS #f
    WRITE #f, "Hello World"
    CLOSE #f

    OPEN "lof_test.tmp" FOR INPUT AS #f
    len1 = LOF(f)
    IF len1 >= 11 THEN
        ConPrint "LOF-PASS len=" & STR$(len1)
    ELSE
        ConPrint "LOF-FAIL len=" & STR$(len1)
    END IF

    pos1 = LOC(f)
    IF pos1 = 0 THEN
        ConPrint "LOC-PASS pos=" & STR$(pos1)
    ELSE
        ConPrint "LOC-FAIL pos=" & STR$(pos1)
    END IF

    SEEK #f, 2
    pos1 = LOC(f)
    IF pos1 = 0 THEN
        ConPrint "SEEK-FUNC-PASS pos=" & STR$(pos1)
    ELSE
        ConPrint "SEEK-FUNC-FAIL pos=" & STR$(pos1)
    END IF
    CLOSE #f
    KILL "lof_test.tmp"
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


