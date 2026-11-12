' batch342_test.bas - NUL-safety probe, round 2.
' batch340 covered WRAP$/UNWRAP$/MCASE$/VARIANT$/VARIANT$$. This batch covers
' the remaining runtime text boundaries that still measured length with C
' strlen: CHRTOUTF8$ / UTF8TOCHR$ byte conversions and the array join.
' Source s = A 00 B 00 C (LEN 5); an embedded NUL in the middle must not
' truncate the string on either leg of the round trip.

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
    LOCAL s AS STRING, r AS STRING, u AS STRING, a AS STRING, b AS STRING
    LOCAL jarr() AS STRING

    s = CHR$(65) + CHR$(0) + CHR$(66) + CHR$(0) + CHR$(67)
    CheckLong LEN(s), 5, "src-len"

    ' --- CHRTOUTF8$: ASCII + embedded NULs map 1:1 to UTF-8 bytes; NUL byte
    '     (U+0000 -> 0x00) must survive, result stays 5 bytes.
    r = CHRTOUTF8$(s)
    CheckLong LEN(r), 5, "chr2utf8-len"
    CheckByte ASC(r, 1), 65, "chr2utf8-b1"
    CheckByte ASC(r, 2), 0,  "chr2utf8-b2"
    CheckByte ASC(r, 3), 66, "chr2utf8-b3"
    CheckByte ASC(r, 4), 0,  "chr2utf8-b4"
    CheckByte ASC(r, 5), 67, "chr2utf8-b5"

    ' --- UTF8TOCHR$: reverse leg of the same 5-byte payload.
    u = UTF8TOCHR$(r)
    CheckLong LEN(u), 5, "utf82chr-len"
    CheckByte ASC(u, 1), 65, "utf82chr-b1"
    CheckByte ASC(u, 2), 0,  "utf82chr-b2"
    CheckByte ASC(u, 3), 66, "utf82chr-b3"
    CheckByte ASC(u, 4), 0,  "utf82chr-b4"
    CheckByte ASC(u, 5), 67, "utf82chr-b5"

    ' --- BUILD$: two NUL-bearing operands concatenated byte-for-byte.
    a = CHR$(65) + CHR$(0) + CHR$(66)
    b = CHR$(67) + CHR$(0) + CHR$(68)
    r = BUILD$(a, b)
    CheckLong LEN(r), 6, "build-len"
    CheckByte ASC(r, 1), 65, "build-b1"
    CheckByte ASC(r, 2), 0,  "build-b2"
    CheckByte ASC(r, 3), 66, "build-b3"
    CheckByte ASC(r, 4), 67, "build-b4"
    CheckByte ASC(r, 5), 0,  "build-b5"
    CheckByte ASC(r, 6), 68, "build-b6"

    ' --- JOIN$: two NUL-bearing array elements joined by "," (0x2C).
    REDIM jarr(1 TO 2)
    jarr(1) = a
    jarr(2) = b
    r = JOIN$(jarr(), ",")
    CheckLong LEN(r), 7, "join-len"
    CheckByte ASC(r, 1), 65, "join-b1"
    CheckByte ASC(r, 2), 0,  "join-b2"
    CheckByte ASC(r, 3), 66, "join-b3"
    CheckByte ASC(r, 4), 44, "join-b4"
    CheckByte ASC(r, 5), 67, "join-b5"
    CheckByte ASC(r, 6), 0,  "join-b6"
    CheckByte ASC(r, 7), 68, "join-b7"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures) + " ==="
    FUNCTION = g_failures
END FUNCTION
