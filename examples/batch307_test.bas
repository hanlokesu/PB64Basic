#COMPILE EXE
#DIM ALL
' batch307_test.bas - release test example (dual-compile: PBWin10 + PB64Basic).
' Covers legacy #8 (TYPE member multi-dim arrays), #15 (FASTPROC),
' #18 (REDIM PRESERVE).  Console via Win32 API (batch 234 pattern).
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB

TYPE B307Rec
    x AS LONG
    arr(1 TO 3, 1 TO 2) AS LONG
END TYPE

FASTPROC B307Add (BYVAL a AS LONG, BYVAL b AS LONG) AS LONG
END FASTPROC = a + b

FUNCTION PBMAIN() AS LONG
    LOCAL r AS B307Rec
    LOCAL ok AS LONG
    LOCAL fails AS LONG
    DIM arr(2) AS LONG
    arr(2) = 41
    REDIM PRESERVE arr(9) AS LONG
    IF arr(2) <> 41 THEN INCR fails
    r.arr(1, 1) = 10
    r.arr(3, 2) = 11
    IF r.arr(1, 1) <> 10 OR r.arr(3, 2) <> 11 THEN INCR fails
    IF B307Add(2, 3) <> 5 THEN INCR fails
    IF fails = 0 THEN ok = 1
    ConPrint "batch307: ok=" & STR$(ok) & " fails=" & STR$(fails) & $CRLF
    FUNCTION = 0
END FUNCTION
