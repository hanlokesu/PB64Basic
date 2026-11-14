' batch351_test.bas - ASC statement type-guard regression (batch 351).
' The ASC assignment on a WSTRING / UDT / numeric left value used to fall
' through an empty codegen arm and be dropped silently; it now fails at
' compile time (negative probe b351_asc_wstring.bas). This sample covers
' the legal STRING and STRING * N paths that must keep working.
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

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL t AS STRING
    LOCAL f AS STRING * 8
    fails = 0

    ' dynamic STRING: first-character replacement
    t = "hello There"
    ASC(t, 1) = 72
    IF t <> "Hello There" THEN fails = fails + 1

    ' dynamic STRING: negative index counts from the end
    t = "abc"
    ASC(t, -1) = 90
    IF t <> "abZ" THEN fails = fails + 1

    ' dynamic STRING: out-of-range index is ignored
    t = "xyz"
    ASC(t, 99) = 65
    IF t <> "xyz" THEN fails = fails + 1

    ' dynamic STRING: empty string is ignored
    t = ""
    ASC(t, 1) = 65
    IF t <> "" THEN fails = fails + 1

    ' fixed-length STRING * N: in-place byte replacement
    f = "abcdefgh"
    ASC(f, 2) = 88
    IF f <> "aXcdefgh" THEN fails = fails + 1

    ConPrint "=== FAILURES: " & STR$(fails)
    FUNCTION = 0
END FUNCTION
