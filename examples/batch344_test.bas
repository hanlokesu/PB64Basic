' batch344_test.bas - #INCLUDE resolution witness for the --include-dir / /I switch.
' The command-line extra include directories are exercised by the three-state probe
' under _bak; this locked example covers the same-directory (parent) resolution path
' every release must keep working: a same-dir _inc.bas defines an equate and a helper
' function that must be visible after #INCLUDE (dual-verified 2026-11-13).

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

SUB CheckNum(BYVAL got AS LONG, BYVAL expected AS LONG, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=" + STR$(got) + " expected=" + STR$(expected)
    END IF
END SUB

#INCLUDE "batch344_inc.bas"

FUNCTION PBMAIN () AS LONG
    LOCAL n AS LONG

    n = IncB344(10)
    CheckNum %B344_CONST, 344, "inc equate"
    CheckNum n, 354, "inc helper"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures) + " ==="
    FUNCTION = g_failures
END FUNCTION
