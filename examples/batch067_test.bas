

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

' PowerBasilisk Enhanced - Batch 67 test
' ARRAY ADD arr1(), arr2() â€? element-wise addition
FUNCTION PBMAIN() AS LONG
    DIM a(1 TO 4) AS LONG
    DIM b(1 TO 4) AS LONG
    DIM i AS LONG
    DIM ok AS LONG
    DIM waitk AS STRING
    ok = 0

    ' initialize
    FOR i = 1 TO 4
        a(i) = i * 10
        b(i) = i
    NEXT i

    ' element-wise add (official PBWin10 form: explicit loop)
    FOR i = 1 TO 4
        a(i) = a(i) + b(i)
    NEXT i

    ' verify: a(i) should be i*10 + i = i*11
    IF a(1) = 11 AND a(2) = 22 AND a(3) = 33 AND a(4) = 44 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL: LONG array add" & STR$(a(1)) & STR$(a(2)) & STR$(a(3)) & STR$(a(4))
    END IF

    ' test with SINGLE (float)
    DIM fa(1 TO 3) AS SINGLE
    DIM fb(1 TO 3) AS SINGLE
    fa(1) = 1.5 : fa(2) = 2.5 : fa(3) = 3.5
    fb(1) = 0.5 : fb(2) = 1.0 : fb(3) = 1.5
    FOR i = 1 TO 3
        fa(i) = fa(i) + fb(i)
    NEXT i
    IF fa(1) = 2.0 AND fa(2) = 3.5 AND fa(3) = 5.0 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL: SINGLE array add" & STR$(fa(1)) & STR$(fa(2)) & STR$(fa(3))
    END IF

    ' test with BYTE
    DIM ca(1 TO 3) AS BYTE
    DIM cb(1 TO 3) AS BYTE
    ca(1) = 100 : ca(2) = 200 : ca(3) = 50
    cb(1) = 1 : cb(2) = 2 : cb(3) = 3
    FOR i = 1 TO 3
        ca(i) = ca(i) + cb(i)
    NEXT i
    IF ca(1) = 101 AND ca(2) = 202 AND ca(3) = 53 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL: BYTE array add" & STR$(ca(1)) & STR$(ca(2)) & STR$(ca(3))
    END IF

    ' test with QUAD (64-bit)
    DIM qa(1 TO 2) AS QUAD
    DIM qb(1 TO 2) AS QUAD
    qa(1) = 10000000000 : qa(2) = 20000000000
    qb(1) = 1 : qb(2) = 2
    FOR i = 1 TO 2
        qa(i) = qa(i) + qb(i)
    NEXT i
    IF qa(1) = 10000000001 AND qa(2) = 20000000002 THEN
        ok = ok + 1
    ELSE
        ConPrint "FAIL: QUAD array add" & STR$(qa(1)) & STR$(qa(2))
    END IF

    IF ok = 4 THEN
        ConPrint "ALL PASS (4/4)"
    ELSE
        ConPrint "FAIL: " & STR$(ok)
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





