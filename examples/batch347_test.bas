' batch347_test.bas - Happy-path witness for the C-family numeric conversions.
' The string-operand rejection is covered by _negative_probes/b347_*_string.bas;
' this file locks the numeric path: every C-family keyword still converts a
' numeric operand, and the value is unchanged.

#COMPILE EXE
#DIM ALL

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
    IF h <> 0 THEN WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

GLOBAL g_failures AS LONG

SUB Fail(BYVAL tag AS STRING)
    g_failures = g_failures + 1
    ConPrint "FAIL " + tag
END SUB

FUNCTION PBMAIN () AS LONG
    IF CINT(3) <> 3 THEN Fail "CINT"
    IF CLNG(3) <> 3 THEN Fail "CLNG"
    IF CDWD(3) <> 3 THEN Fail "CDWD"
    IF CLNGINT(3) <> 3 THEN Fail "CLNGINT"
    IF CUINT(3) <> 3 THEN Fail "CUINT"
    IF CULNG(3) <> 3 THEN Fail "CULNG"
    IF CQUAD(3) <> 3 THEN Fail "CQUAD"
    IF CQUD(3) <> 3 THEN Fail "CQUD"
    IF CDWORD(3) <> 3 THEN Fail "CDWORD"
    IF CBYTE(3) <> 3 THEN Fail "CBYTE"
    IF CBYT(3) <> 3 THEN Fail "CBYT"
    IF CWORD(3) <> 3 THEN Fail "CWORD"
    IF CWRD(3) <> 3 THEN Fail "CWRD"
    IF CDBL(3) <> 3.0 THEN Fail "CDBL"
    IF CEXT(3) <> 3.0 THEN Fail "CEXT"
    IF CSNG(3) <> 3.0 THEN Fail "CSNG"
    IF CFLT(3) <> 3.0 THEN Fail "CFLT"
    IF CCUR(3) <> 3.0 THEN Fail "CCUR"
    IF CCUX(3) <> 3.0 THEN Fail "CCUX"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures) + " ==="
    FUNCTION = g_failures
END FUNCTION
