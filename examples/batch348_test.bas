#COMPILE EXE
#DIM ALL

' batch 348 self-check:
'   - conversion guards still accept legitimate numeric operands (array element,
'     struct field, Variant, a real zero-argument function);
'   - DIM ... AS LONG overrides an inherited THREADED / GLOBAL element type, while
'     a bare DIM keeps the declared type (i16 wrap proves it);
'   - EXTRACT$ and MEMORY FILL use the BSTR byte length, so embedded NUL survives;
'   - repeated thread create/exit with a THREADED array stays correct (the block
'     release itself is measured externally by _bak/_b348_d4.py, whose control
'     build with FLS registration disabled leaks ~160 MB and is detected).

TYPE T348
    x AS LONG
    y AS LONG
END TYPE

THREADED tInt%()
THREADED tStr() AS STRING
THREADED tBig() AS LONG
GLOBAL gStrArr() AS STRING
GLOBAL gIntArr() AS INTEGER
DIM gFlags(1 TO 4) AS LONG
GLOBAL g_fail AS LONG

FUNCTION Seven() AS LONG
    FUNCTION = 7
END FUNCTION

SUB Bump()
    g_fail = g_fail + 1
END SUB

SUB CheckLong(BYVAL got AS LONG, BYVAL want AS LONG, BYVAL what AS STRING)
    IF got <> want THEN
        PRINT "FAIL "; what; " got="; got; " want="; want
        Bump
    END IF
END SUB

THREAD FUNCTION FillWorker(BYVAL tid AS LONG) AS LONG
    DIM tBig(1 TO 100000)
    LOCAL k AS LONG, ok AS LONG
    ok = 1
    FOR k = 1 TO 100000
        tBig(k) = tid
    NEXT k
    IF tBig(1) <> tid THEN ok = 0
    IF tBig(100000) <> tid THEN ok = 0
    gFlags(tid) = ok
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL i AS LONG, r AS LONG, n AS LONG, rc AS LONG, waits AS LONG, total AS LONG
    LOCAL t AS T348
    LOCAL var AS VARIANT
    LOCAL s AS STRING
    LOCAL hh(1 TO 4) AS DWORD

    DIM a(1 TO 3) AS LONG
    a(1) = 11
    a(2) = 22
    a(3) = 33
    CheckLong CINT(a(1)), 11, "array element CINT"
    CheckLong CLNG(a(2)), 22, "array element CLNG"
    CheckLong CINT(Seven()), 7, "zero-arg function"

    t.x = 41
    t.y = 42
    CheckLong CINT(t.x), 41, "struct field CINT"
    CheckLong CLNG(t.y), 42, "struct field CLNG"

    var = 55
    CheckLong CINT(var), 55, "variant CINT"
    IF LEN(STR$(a(3))) < 2 THEN Bump

    ' D2: explicit AS LONG overrides the inherited Integer element type
    DIM tInt(1 TO 3) AS LONG
    tInt(1) = 70000
    CheckLong tInt(1), 70000, "THREADED Integer -> AS LONG"

    ' D2: explicit AS LONG overrides the inherited STRING (Ptr) element type
    DIM tStr(1 TO 3) AS LONG
    tStr(1) = 12345
    CheckLong tStr(1), 12345, "THREADED STRING -> AS LONG"

    ' D2: global STRING array pending a later DIM, explicit AS LONG wins
    DIM gStrArr(1 TO 3) AS LONG
    gStrArr(1) = 22222
    CheckLong gStrArr(1), 22222, "GLOBAL STRING -> AS LONG"

    ' D2: bare DIM keeps the declared Integer (70000 wraps to 4464 in i16)
    DIM gIntArr(1 TO 3)
    gIntArr(1) = 70000
    CheckLong gIntArr(1), 4464, "bare DIM keeps Integer"

    ' E1: EXTRACT$ and MEMORY FILL use the BSTR byte length (embedded NUL kept)
    s = "AB" + CHR$(0) + "CD"
    CheckLong LEN(EXTRACT$(3, s, "ZZ")), 3, "EXTRACT$ embedded NUL"

    DIM mfbuf(1 TO 9) AS BYTE
    MEMORY FILL VARPTR(mfbuf(1)), 9, "A" + CHR$(0) + "B"
    IF mfbuf(1) <> 65 THEN Bump
    IF mfbuf(2) <> 0 THEN Bump
    IF mfbuf(3) <> 66 THEN Bump
    IF mfbuf(4) <> 65 THEN Bump

    ' D4: repeated create/exit of threads that DIM a THREADED array stays correct
    FOR r = 1 TO 8
        FOR i = 1 TO 4
            gFlags(i) = 0
        NEXT i
        FOR i = 1 TO 4
            THREAD CREATE FillWorker(i) TO hh(i)
        NEXT i
        waits = 0
        DO WHILE waits < 400
            total = gFlags(1) + gFlags(2) + gFlags(3) + gFlags(4)
            IF total = 4 THEN EXIT DO
            SLEEP 5
            waits = waits + 1
        LOOP
        FOR i = 1 TO 4
            THREAD CLOSE hh(i) TO rc
        NEXT i
        total = gFlags(1) + gFlags(2) + gFlags(3) + gFlags(4)
        IF total <> 4 THEN Bump
    NEXT r

    PRINT "=== FAILURES: "; g_fail
    FUNCTION = 0
END FUNCTION