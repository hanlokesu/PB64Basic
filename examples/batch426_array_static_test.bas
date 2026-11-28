' =====================================================================
' PB64Basic Enhanced - batch 426 - STATIC array persistence / REDIM list
'   Regression for 0263 Reverse_ASM_Hutch: STATIC arrays must survive
'   callback frames, REDIM a(),b() must dimension both arrays, ARRAYATTR
'   must report the live element total, and POKE/POKE$ must accept a full
'   Long Pointer without inttoptr(ptr).
'   === FAILURES: 0
' =====================================================================
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

GLOBAL fails AS LONG

SUB StaticProbe(BYVAL phase AS LONG, BYREF outv AS LONG, BYREF ub AS LONG)
    STATIC s() AS LONG
    IF phase = 1 THEN
        REDIM s(3)
        s(0) = 42
    ELSE
        outv = s(0)
        ub = UBOUND(s)
    END IF
END SUB

SUB AttrProbe(x() AS LONG, BYREF total AS LONG, BYREF ub AS LONG)
    total = ARRAYATTR(x(), 4)
    ub = UBOUND(x)
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL a() AS LONG, b() AS LONG
    LOCAL undimmed() AS LONG
    LOCAL p AS LONG POINTER
    LOCAL v AS LONG, ub AS LONG, t AS LONG

    fails = 0

    ' Undimensioned DIM a(): ARRAYATTR must say 0, not 1 (empty product).
    IF ARRAYATTR(undimmed(), 4) <> 0 THEN fails = fails + 1
    IF UBOUND(undimmed) <> -1 THEN fails = fails + 1
    CALL AttrProbe(undimmed(), t, ub)
    IF t <> 0 THEN fails = fails + 1
    IF ub <> -1 THEN fails = fails + 1

    REDIM a(3), b(5)
    IF UBOUND(a) <> 3 THEN fails = fails + 1
    IF UBOUND(b) <> 5 THEN fails = fails + 1
    IF ARRAYATTR(a(), 4) <> 4 THEN fails = fails + 1
    IF ARRAYATTR(b(), 4) <> 6 THEN fails = fails + 1

    p = VARPTR(a(0))
    POKE LONG, p, 99
    IF a(0) <> 99 THEN fails = fails + 1

    ' String POKE must accept a full Long Pointer too (x64: no inttoptr ptr).
    POKE$ p, "WXYZ"
    IF PEEK(BYTE, p) <> 87 THEN fails = fails + 1

    ' Multi-variable REDIM PRESERVE keeps both bounds and existing data.
    a(1) = 7
    REDIM PRESERVE a(5), b(7)
    IF UBOUND(a) <> 5 THEN fails = fails + 1
    IF UBOUND(b) <> 7 THEN fails = fails + 1
    IF a(1) <> 7 THEN fails = fails + 1

    CALL StaticProbe(1, v, ub)
    CALL StaticProbe(2, v, ub)
    IF v <> 42 THEN fails = fails + 1
    IF ub <> 3 THEN fails = fails + 1

    CALL AttrProbe(a(), t, ub)
    IF t <> 6 THEN fails = fails + 1
    IF ub <> 5 THEN fails = fails + 1

    ConPrint "static=" & STR$(v) & " ub=" & STR$(ub) & " total=" & STR$(t) & CHR$(13, 10)
    ConPrint "=== FAILURES: " & STR$(fails) & CHR$(13, 10)
    FUNCTION = 0
END FUNCTION
