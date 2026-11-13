' batch343_test.bas - Compile-time type guards for the numeric-to-string family.
' STR$/CSTR$, HEX$, OCT$, BIN$, DEC$ reject a BSTR/pointer operand at compile
' time (PBWin10 Error 420); the string-operand rejection itself is covered by
' the negative probes in _negative_probes/b323_*_string.bas. This file locks the
' happy path: numeric operands still render exactly as PBWin10 (dual-verified
' 2026-11-13).

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

SUB CheckStr(BYVAL got AS STRING, BYVAL expected AS STRING, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=[" + got + "] expected=[" + expected + "]"
    END IF
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL r AS STRING
    LOCAL n AS LONG

    n = 42
    r = STR$(n)
    CheckStr r, " 42", "str 42"
    n = -7
    r = STR$(n)
    CheckStr r, "-7", "str -7"

    r = HEX$(255)
    CheckStr r, "FF", "hex 255"
    r = HEX$(-1)
    CheckStr r, "FFFFFFFFFFFFFFFF", "hex -1"

    r = OCT$(8)
    CheckStr r, "10", "oct 8"

    r = BIN$(5)
    CheckStr r, "101", "bin 5"
    r = BIN$(0)
    CheckStr r, "0", "bin 0"

    r = DEC$(255)
    CheckStr r, "255", "dec 255"
    r = DEC$(-7)
    CheckStr r, "-7", "dec -7"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures) + " ==="
    FUNCTION = g_failures
END FUNCTION
