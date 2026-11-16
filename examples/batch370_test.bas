#COMPILE EXE
#DIM ALL

' batch 370 self-check: THREADED array runtime DIM bounds.
'   - WorkerC dimensions tB(1 TO n) with n = 7.
'   - WorkerD dimensions the same THREADED array with n = 11.
'   - Each thread reads its own LBOUND/UBOUND/ARRAYATTR total and values.
'   - The main thread has not dimensioned tB yet, so UBOUND(tB()) must be -1;
'     it then dimensions tB(1 TO 5) and sees its own shape.

THREADED tB() AS LONG
DIM gLo(1 TO 2) AS LONG
DIM gHi(1 TO 2) AS LONG
DIM gTotal(1 TO 2) AS LONG
DIM gDone(1 TO 2) AS LONG
DIM gFails(1 TO 2) AS LONG

SUB Bump(BYVAL tid AS LONG)
    gFails(tid) = gFails(tid) + 1
END SUB

THREAD FUNCTION WorkerC(BYVAL tid AS LONG) AS LONG
    LOCAL i AS LONG
    LOCAL n AS LONG
    n = 7
    DIM tB(1 TO n)
    FOR i = 1 TO 7
        tB(i) = i * 2
    NEXT i
    gLo(tid) = LBOUND(tB())
    gHi(tid) = UBOUND(tB())
    gTotal(tid) = ARRAYATTR(tB(), 4)
    IF tB(1) <> 2 THEN Bump tid
    IF tB(7) <> 14 THEN Bump tid
    gDone(tid) = 1
END FUNCTION

THREAD FUNCTION WorkerD(BYVAL tid AS LONG) AS LONG
    LOCAL i AS LONG
    LOCAL n AS LONG
    n = 11
    DIM tB(1 TO n)
    FOR i = 1 TO 11
        tB(i) = i * 3
    NEXT i
    gLo(tid) = LBOUND(tB())
    gHi(tid) = UBOUND(tB())
    gTotal(tid) = ARRAYATTR(tB(), 4)
    IF tB(1) <> 3 THEN Bump tid
    IF tB(11) <> 33 THEN Bump tid
    gDone(tid) = 1
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL hC AS DWORD, hD AS DWORD
    LOCAL waits AS LONG

    IF UBOUND(tB()) <> -1 THEN Bump 1

    THREAD CREATE WorkerC(1) TO hC
    THREAD CREATE WorkerD(2) TO hD
    waits = 0
    DO WHILE waits < 200
        IF gDone(1) = 1 AND gDone(2) = 1 THEN EXIT DO
        SLEEP 5
        waits = waits + 1
    LOOP
    THREAD CLOSE hC TO hC
    THREAD CLOSE hD TO hD

    IF gDone(1) <> 1 THEN Bump 1
    IF gDone(2) <> 1 THEN Bump 2
    IF gLo(1) <> 1 THEN Bump 1
    IF gHi(1) <> 7 THEN Bump 1
    IF gTotal(1) <> 7 THEN Bump 1
    IF gLo(2) <> 1 THEN Bump 2
    IF gHi(2) <> 11 THEN Bump 2
    IF gTotal(2) <> 11 THEN Bump 2

    DIM tB(1 TO 5)
    tB(1) = 41
    tB(5) = 45
    IF LBOUND(tB()) <> 1 THEN Bump 1
    IF UBOUND(tB()) <> 5 THEN Bump 1
    IF ARRAYATTR(tB(), 4) <> 5 THEN Bump 1
    IF tB(1) <> 41 THEN Bump 1
    IF tB(5) <> 45 THEN Bump 1

    PRINT "lo/hi C="; gLo(1); "/"; gHi(1); " total="; gTotal(1)
    PRINT "lo/hi D="; gLo(2); "/"; gHi(2); " total="; gTotal(2)
    PRINT "=== FAILURES: "; gFails(1) + gFails(2)
    FUNCTION = 0
END FUNCTION
