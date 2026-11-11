' batch340_test.bas - NUL-safety probe for the five length-prefix fixes
' WRAP$ / UNWRAP$ / MCASE$ / VARIANT$ / VARIANT$$ must read the BSTR byte
' length from the prefix, not strlen: an embedded NUL in the middle must not
' truncate the string. Source s = A 00 B 00 C (LEN 5).

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
    LOCAL v AS VARIANT
    LOCAL w AS WSTRING

    s = CHR$(65) + CHR$(0) + CHR$(66) + CHR$(0) + CHR$(67)
    CheckLong LEN(s), 5, "src-len"

    ' WRAP$: brackets plus the full 5-byte payload -> 7 bytes
    r = WRAP$(s, "[", "]")
    CheckLong LEN(r), 7, "wrap-len"
    CheckByte ASC(r, 1), 91, "wrap-b1"
    CheckByte ASC(r, 2), 65, "wrap-b2"
    CheckByte ASC(r, 3), 0, "wrap-b3"
    CheckByte ASC(r, 4), 66, "wrap-b4"
    CheckByte ASC(r, 5), 0, "wrap-b5"
    CheckByte ASC(r, 6), 67, "wrap-b6"
    CheckByte ASC(r, 7), 93, "wrap-b7"

    ' UNWRAP$: no matching markers -> full 5-byte string back
    r = UNWRAP$(s, "[", "]")
    CheckLong LEN(r), 5, "unwrap-len"
    CheckByte ASC(r, 1), 65, "unwrap-b1"
    CheckByte ASC(r, 3), 66, "unwrap-b3"
    CheckByte ASC(r, 5), 67, "unwrap-b5"

    ' MCASE$: length preserved, embedded NULs intact
    r = MCASE$(s)
    CheckLong LEN(r), 5, "mcase-len"
    CheckByte ASC(r, 2), 0, "mcase-b2"
    CheckByte ASC(r, 4), 0, "mcase-b4"

    ' VARIANT$: ANSI form keeps the embedded NUL bytes
    v = s
    r = VARIANT$(v)
    CheckLong LEN(r), 5, "varstr-len"
    CheckByte ASC(r, 1), 65, "varstr-b1"
    CheckByte ASC(r, 2), 0, "varstr-b2"
    CheckByte ASC(r, 3), 66, "varstr-b3"
    CheckByte ASC(r, 4), 0, "varstr-b4"
    CheckByte ASC(r, 5), 67, "varstr-b5"

    ' VARIANT$$: wide form keeps 5 wide characters
    w = VARIANT$$(v)
    CheckLong LEN(w), 5, "varwstr-len"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures) + " ==="
    FUNCTION = g_failures
END FUNCTION
