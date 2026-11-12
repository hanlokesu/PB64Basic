' batch343_test.bas - NUL-safety probe, round 3.
' CLIP$ / SHRINK$ / REMOVE$ / RETAIN$ / REMAIN$ must measure the BSTR byte
' length (prefix), not C strlen.  Source s = A 00 B 00 C (LEN 5).

#COMPILE EXE
#DIM ALL

DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
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

SUB CheckLong(BYVAL got AS LONG, BYVAL expected AS LONG, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=" + FORMAT$(got) + " expected=" + FORMAT$(expected)
    END IF
END SUB

SUB CheckByte(BYVAL got AS LONG, BYVAL expected AS LONG, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=" + FORMAT$(got) + " expected=" + FORMAT$(expected)
    END IF
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL s AS STRING, r AS STRING

    s = CHR$(65) + CHR$(0) + CHR$(66) + CHR$(0) + CHR$(67)
    CheckLong LEN(s), 5, "src-len"

    ' CLIP$ LEFT 1: drop leading A -> NUL B NUL C
    r = CLIP$(LEFT s, 1)
    CheckLong LEN(r), 4, "clipL-len"
    CheckByte ASC(r, 1), 0,  "clipL-b1"
    CheckByte ASC(r, 2), 66, "clipL-b2"
    CheckByte ASC(r, 3), 0,  "clipL-b3"
    CheckByte ASC(r, 4), 67, "clipL-b4"

    ' CLIP$ RIGHT 1: drop trailing C -> A NUL B NUL
    r = CLIP$(RIGHT s, 1)
    CheckLong LEN(r), 4, "clipR-len"
    CheckByte ASC(r, 1), 65, "clipR-b1"
    CheckByte ASC(r, 2), 0,  "clipR-b2"
    CheckByte ASC(r, 3), 66, "clipR-b3"
    CheckByte ASC(r, 4), 0,  "clipR-b4"

    ' CLIP$ MID s,2,1: drop position 2 (the first NUL) -> A B NUL C
    r = CLIP$(MID s, 2, 1)
    CheckLong LEN(r), 4, "clipM-len"
    CheckByte ASC(r, 1), 65, "clipM-b1"
    CheckByte ASC(r, 2), 66, "clipM-b2"
    CheckByte ASC(r, 3), 0,  "clipM-b3"
    CheckByte ASC(r, 4), 67, "clipM-b4"

    ' REMOVE$ "B": drop B -> A NUL NUL C
    r = REMOVE$(s, "B")
    CheckLong LEN(r), 4, "rem-len"
    CheckByte ASC(r, 1), 65, "rem-b1"
    CheckByte ASC(r, 2), 0,  "rem-b2"
    CheckByte ASC(r, 3), 0,  "rem-b3"
    CheckByte ASC(r, 4), 67, "rem-b4"

    ' RETAIN$ ANY "AC": keep only A and C -> A C
    r = RETAIN$(s, ANY "AC")
    CheckLong LEN(r), 2, "ret-len"
    CheckByte ASC(r, 1), 65, "ret-b1"
    CheckByte ASC(r, 2), 67, "ret-b2"

    ' REMAIN$ "B": portion after first B -> NUL C
    r = REMAIN$(s, "B")
    CheckLong LEN(r), 2, "rmn-len"
    CheckByte ASC(r, 1), 0,  "rmn-b1"
    CheckByte ASC(r, 2), 67, "rmn-b2"

    ' SHRINK$: NUL is not whitespace, no run collapses -> unchanged A NUL B NUL C
    r = SHRINK$(s)
    CheckLong LEN(r), 5, "shr-len"
    CheckByte ASC(r, 1), 65, "shr-b1"
    CheckByte ASC(r, 2), 0,  "shr-b2"
    CheckByte ASC(r, 3), 66, "shr-b3"
    CheckByte ASC(r, 4), 0,  "shr-b4"
    CheckByte ASC(r, 5), 67, "shr-b5"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures) + " ==="
    FUNCTION = g_failures
END FUNCTION
