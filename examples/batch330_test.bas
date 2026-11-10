#COMPILE EXE

' batch330 self-check: PEEK/PEEK$ CHM conformance (the CLI command-line fix is
' covered by the release gates and the forum repro, not by an in-program check).
'   1) Numeric PEEK supports every CHM data type: BYTE/WORD/DWORD/INTEGER/LONG/
'      QUAD/SINGLE/DOUBLE plus EXT/CUR/CUX. The EXT/CUR/CUX forms previously
'      fell through to the BYTE default and read only one byte.
'   2) PEEK$(address, count) reads a fixed number of bytes; the two arguments
'      are the address and the byte count - the original two-argument form.
' Headless only: every assertion is counted, output goes through ConPrint,
' no WAITKEY / MSGBOX / dialog.

DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverridden AS LONG) AS LONG

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
        ConPrint "FAIL " + tag + " got=" + FORMAT$(got) + " expected=" + FORMAT$(expected) + $CRLF
    END IF
END SUB

SUB CheckNum(BYVAL got AS DOUBLE, BYVAL expected AS DOUBLE, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=" + FORMAT$(got) + " expected=" + FORMAT$(expected) + $CRLF
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL bv AS BYTE
    LOCAL wv AS WORD
    LOCAL iv AS INTEGER
    LOCAL lv AS LONG
    LOCAL dv AS DWORD
    LOCAL qv AS QUAD
    LOCAL sv AS SINGLE
    LOCAL db AS DOUBLE
    LOCAL cv AS CUR
    LOCAL xc AS CUX
    LOCAL rb AS DOUBLE
    LOCAL raw AS STRING

    bv = 65
    wv = 1234
    iv = -1234
    lv = 100000
    dv = 100000
    qv = 123456789
    sv = 1.5
    db = 3.25
    cv = 9.5
    xc = 7.25

    ' --- 1) numeric PEEK across every CHM data type ---
    CheckLong PEEK(BYTE, VARPTR(bv)), 65, "BYTE"
    CheckLong PEEK(WORD, VARPTR(wv)), 1234, "WORD"
    CheckLong PEEK(INTEGER, VARPTR(iv)), -1234, "INTEGER"
    CheckLong PEEK(LONG, VARPTR(lv)), 100000, "LONG"
    CheckLong PEEK(DWORD, VARPTR(dv)), 100000, "DWORD"
    CheckLong PEEK(QUAD, VARPTR(qv)), 123456789, "QUAD"

    rb = PEEK(SINGLE, VARPTR(sv))
    CheckNum rb, 1.5, "SINGLE"
    rb = PEEK(DOUBLE, VARPTR(db))
    CheckNum rb, 3.25, "DOUBLE"
    rb = PEEK(EXT, VARPTR(db))
    CheckNum rb, 3.25, "EXT"
    rb = PEEK(CUR, VARPTR(cv))
    CheckNum rb, 9.5, "CUR"
    rb = PEEK(CUX, VARPTR(xc))
    CheckNum rb, 7.25, "CUX"

    ' --- 2) PEEK$(address, count): two-argument byte read ---
    raw = PEEK$(VARPTR(bv), 4)
    CheckLong LEN(raw), 4, "PEEK$ length"
    CheckLong ASC(raw, 1), 65, "PEEK$ first byte"

    ConPrint "=== FAILURES: " + FORMAT$(g_failures) + $CRLF
    FUNCTION = g_failures
END FUNCTION
