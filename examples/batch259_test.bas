' batch259_test.bas - ARRAY SORT/SCAN COLLATE + MID$ lvalue (batch 259; PBWin-measured values)
' (rebuilt from the batch's real optimization topic; see SKILL.md)
#COMPILE EXE
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
FUNCTION PBMAIN() AS LONG
    LOCAL ok AS LONG
    LOCAL i AS LONG
    LOCAL r AS LONG
    LOCAL c AS STRING
    LOCAL s AS STRING
    DIM sf$(1 TO 4)
    DIM su$(1 TO 3)
    DIM cl$(1 TO 3)
    ok = 0
    sf$(1) = "New York 10001"
    sf$(2) = "Boston 02110"
    sf$(3) = "Dallas 75201"
    sf$(4) = "Denver 80210"
    ARRAY SORT sf$()
    IF sf$(1) = "Boston 02110" AND sf$(2) = "Dallas 75201" AND sf$(3) = "Denver 80210" AND sf$(4) = "New York 10001" THEN ok = ok OR 1
    su$(1) = "Cathy"
    su$(2) = "bob"
    su$(3) = "Ann"
    ARRAY SORT su$(), COLLATE UCASE, DESCEND
    IF su$(1) = "Cathy" AND su$(2) = "bob" AND su$(3) = "Ann" THEN ok = ok OR 2
    c = ""
    FOR i = 0 TO 255
        c = c + CHR$(i)
    NEXT
    MID$(c, ASC("a") + 1, 1) = CHR$(200)
    cl$(1) = "alpha"
    cl$(2) = "bravo"
    cl$(3) = "charlie"
    ARRAY SORT cl$(), COLLATE c
    IF cl$(1) = "bravo" AND cl$(2) = "charlie" AND cl$(3) = "alpha" THEN ok = ok OR 4
    ARRAY SCAN cl$(), COLLATE c, > "bravo", TO r
    IF r = 2 THEN ok = ok OR 8
    s = "hello"
    MID$(s, 2, 1) = "E"
    IF s = "hEllo" THEN ok = ok OR 16
    IF ok = 31 THEN
        ConPrint "batch259 test ok"
    ELSE
        ConPrint "batch259 FAILURES: " & STR$(ok)
    END IF
    FUNCTION = 0
END FUNCTION

