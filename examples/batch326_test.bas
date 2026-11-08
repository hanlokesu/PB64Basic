#COMPILE EXE

' batch 326 self-check: U / V / W keyword family fixes audited against the CHW.
' Headless only: every assertion is counted, results go through ConPrint, no
' WAITKEY / MSGBOX / dialog. Each checked value was verified byte-for-byte
' against the PBWin 10 reference compiler during the family audit.

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

SUB CheckStr(BYVAL got AS STRING, BYVAL expected AS STRING, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=[" + got + "] expected=[" + expected + "]"
    END IF
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL i AS LONG, cnt AS LONG
    LOCAL v AS DOUBLE, d AS LONG, u AS LONG
    LOCAL r AS DOUBLE
    LOCAL s AS STRING
    DIM a(1 TO 5) AS LONG
    DIM b() AS LONG
    DIM w AS WORD

    ' --- U family ---
    CheckStr UCASE$("abc"), "ABC", "ucase"
    CheckLong UBOUND(a), 5, "ubound"
    CheckLong LBOUND(a), 1, "lbound"
    CheckLong UBOUND(b), -1, "ubound-undim"
    CheckLong LBOUND(b), 0, "lbound-undim"
    CheckStr USING$("##", 5), " 5", "using"

    ' --- V family: VAL function (returns Double; convert before integer compare) ---
    r = VAL("123")
    CheckLong CLNG(r), 123, "val-dec"
    r = VAL("&HFF")
    CheckLong CLNG(r), 255, "val-hex"

    ' --- V family: VAL statement (CHW Val_statement.htm) ---
    s = "  123abc"
    VAL s TO v, d, u
    CheckLong CLNG(v), 123, "valstmt-v"
    CheckLong d, 3, "valstmt-digits"
    CheckLong u, 3, "valstmt-unused"
    s = "abc"
    VAL s TO v, d
    CheckLong CLNG(v), 0, "valstmt-v2"
    CheckLong d, 0, "valstmt-d2"

    ' --- V family: VERIFY ---
    CheckLong VERIFY("123.65", "0123456789"), 4, "verify-normal"
    CheckLong VERIFY(0, "123.65", "0123456789"), 0, "verify-start0"

    ' --- W family: WORD (16-bit unsigned, wraps at 65535) ---
    w = 65535
    CheckLong w, 65535, "word-max"
    w = w + 1
    CheckLong w, 0, "word-wrap"

    ' --- W family: WHILE / WEND ---
    cnt = 0
    i = 1
    WHILE i <= 5
        cnt = cnt + 1
        i = i + 1
    WEND
    CheckLong cnt, 5, "while"

    ' --- W family: WRAP / UNWRAP ---
    CheckStr WRAP$("MyWord", "<", ">"), "<MyWord>", "wrap"
    CheckStr UNWRAP$("<MyWord>", "<", ">"), "MyWord", "unwrap"
    CheckStr UNWRAP$("abc", "<", ">"), "abc", "unwrap-nomatch"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures)
    FUNCTION = 0
END FUNCTION
