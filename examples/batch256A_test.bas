' batch256A_test.bas - CHW A-family dual-run probe: ARRAY ASSIGN spread /
' ASC statement / ARRAYATTR / LBOUND UBOUND (batch 256A).
' Assertion form: ok accumulates bit flags; final line stable for regression.
' ConPrint template from batch177_test.bas (verified MATCH).
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
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
FUNCTION PBMAIN() AS LONG
    LOCAL i AS LONG
    LOCAL ok AS LONG
    ok = 0

    ' ===== 1. ARRAY ASSIGN spread (CHW ARRAY_ASSIGN_statement.htm) =====
    DIM x(0 TO 9) AS LONG
    ARRAY ASSIGN x() = 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
    IF x(0) = 1 AND x(1) = 2 AND x(2) = 3 AND x(3) = 4 AND x(4) = 5 _
       AND x(5) = 6 AND x(6) = 7 AND x(7) = 8 AND x(8) = 9 AND x(9) = 10 THEN ok = ok + 1

    ' spread with expressions as values
    DIM y(0 TO 3) AS LONG
    ARRAY ASSIGN y() = 1 + 1, 3 * 3, 4, 5
    IF y(0) = 2 AND y(1) = 9 AND y(2) = 4 AND y(3) = 5 THEN ok = ok + 2

    ' spread starts at the FIRST element (nonzero lower bound)
    DIM a(5 TO 9) AS LONG
    ARRAY ASSIGN a() = 11, 22, 33, 44, 55
    IF a(5) = 11 AND a(6) = 22 AND a(7) = 33 AND a(8) = 44 AND a(9) = 55 THEN ok = ok + 4

    ' spread on a string array
    DIM s(0 TO 2) AS STRING
    ARRAY ASSIGN s() = "alpha", "beta", "gamma"
    IF s(0) = "alpha" AND s(1) = "beta" AND s(2) = "gamma" THEN ok = ok + 8

    ' ===== 2. ASC statement (CHW ASC_statement.htm) =====
    LOCAL t AS STRING
    t = "hello There"
    ASC(t, 1) = 72
    IF t = "Hello There" THEN ok = ok + 16

    t = "abc"
    ASC(t, -1) = 90
    IF t = "abZ" THEN ok = ok + 32

    t = "xyz"
    ASC(t, 99) = 65
    IF t = "xyz" THEN ok = ok + 64

    t = ""
    ASC(t, 1) = 65
    IF t = "" THEN ok = ok + 128

    ' ===== 3. ARRAYATTR (CHW ARRAYATTR_function.htm) =====
    DIM z(0 TO 3, 0 TO 4, 0 TO 5) AS DOUBLE
    IF ARRAYATTR(z(), 0) = -1 THEN ok = ok + 256
    IF ARRAYATTR(z(), 1) = 11 THEN ok = ok + 512
    IF ARRAYATTR(z(), 2) = 0 THEN ok = ok + 1024
    IF ARRAYATTR(z(), 3) = 3 THEN ok = ok + 2048
    IF ARRAYATTR(z(), 4) = 120 THEN ok = ok + 4096
    IF ARRAYATTR(z(), 5) = 8 THEN ok = ok + 8192

    DIM b(5 TO 9) AS LONG
    IF ARRAYATTR(b(), 1) = 5 AND ARRAYATTR(b(), 3) = 1 _
       AND ARRAYATTR(b(), 4) = 5 AND ARRAYATTR(b(), 5) = 4 THEN ok = ok + 16384

    ' ===== 4. LBOUND / UBOUND (CHW See also, batch 256A) =====
    IF LBOUND(b()) = 5 AND UBOUND(b()) = 9 THEN ok = ok + 32768
    DIM m(1 TO 3, 10 TO 12) AS INTEGER
    IF LBOUND(m(), 1) = 1 AND UBOUND(m(), 1) = 3 _
       AND LBOUND(m(), 2) = 10 AND UBOUND(m(), 2) = 12 THEN ok = ok + 65536

    ConPrint "batch256A ok=" & STR$(ok)
    IF ok = 131071 THEN
        ConPrint "batch256A test ok"
    ELSE
        ConPrint "batch256A test FAILED"
    END IF
END FUNCTION
