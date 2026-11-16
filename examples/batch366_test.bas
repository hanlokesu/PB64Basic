#COMPILE EXE
#DIM ALL

' batch 366 self-check: THREADED array "per-thread shape" runtime model.
'   - WorkerA dimensions tA(1 TO 10), WorkerB dimensions tA(1 TO 20).
'   - Each thread sees its own LBOUND/UBOUND, ARRAYATTR(3) dimension count,
'     ARRAYATTR(4) element total and element values.
'   - REDIM PRESERVE inside each thread keeps that thread's prefix and grows
'     only that thread's descriptor.
'   - The main thread then dimensions the same THREADED array with a third
'     shape and verifies it independently.

THREADED tA() AS LONG
DIM gLo(1 TO 2) AS LONG
DIM gHi(1 TO 2) AS LONG
DIM gDims(1 TO 2) AS LONG
DIM gTotal(1 TO 2) AS LONG
DIM gValues(1 TO 2) AS LONG
DIM gDone(1 TO 2) AS LONG
DIM gFails(1 TO 2) AS LONG

SUB Bump(BYVAL tid AS LONG)
    gFails(tid) = gFails(tid) + 1
END SUB

THREAD FUNCTION WorkerA(BYVAL tid AS LONG) AS LONG
    LOCAL i AS LONG
    DIM tA(1 TO 10)
    FOR i = 1 TO 10
        tA(i) = i * 3
    NEXT i
    gLo(tid) = LBOUND(tA())
    gHi(tid) = UBOUND(tA())
    gDims(tid) = ARRAYATTR(tA(), 3)
    gTotal(tid) = ARRAYATTR(tA(), 4)
    IF tA(1) <> 3 THEN Bump tid
    IF tA(10) <> 30 THEN Bump tid
    REDIM PRESERVE tA(1 TO 14)
    IF tA(1) <> 3 THEN Bump tid
    IF tA(10) <> 30 THEN Bump tid
    IF tA(14) <> 0 THEN Bump tid
    gHi(tid) = UBOUND(tA())
    gTotal(tid) = ARRAYATTR(tA(), 4)
    gValues(tid) = 1
    gDone(tid) = 1
END FUNCTION

THREAD FUNCTION WorkerB(BYVAL tid AS LONG) AS LONG
    LOCAL i AS LONG
    DIM tA(1 TO 20)
    FOR i = 1 TO 20
        tA(i) = i * 5
    NEXT i
    gLo(tid) = LBOUND(tA())
    gHi(tid) = UBOUND(tA())
    gDims(tid) = ARRAYATTR(tA(), 3)
    gTotal(tid) = ARRAYATTR(tA(), 4)
    IF tA(1) <> 5 THEN Bump tid
    IF tA(20) <> 100 THEN Bump tid
    REDIM PRESERVE tA(1 TO 24)
    IF tA(1) <> 5 THEN Bump tid
    IF tA(20) <> 100 THEN Bump tid
    IF tA(24) <> 0 THEN Bump tid
    gHi(tid) = UBOUND(tA())
    gTotal(tid) = ARRAYATTR(tA(), 4)
    gValues(tid) = 2
    gDone(tid) = 1
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL hA AS DWORD, hB AS DWORD
    LOCAL waits AS LONG, i AS LONG

    THREAD CREATE WorkerA(1) TO hA
    THREAD CREATE WorkerB(2) TO hB
    waits = 0
    DO WHILE waits < 200
        IF gDone(1) = 1 AND gDone(2) = 1 THEN EXIT DO
        SLEEP 5
        waits = waits + 1
    LOOP
    THREAD CLOSE hA TO hA
    THREAD CLOSE hB TO hB

    IF gDone(1) <> 1 THEN Bump 1
    IF gDone(2) <> 1 THEN Bump 2

    ' WorkerA: 1 TO 10 -> PRESERVE 1 TO 14
    IF gLo(1) <> 1 THEN Bump 1
    IF gHi(1) <> 14 THEN Bump 1
    IF gDims(1) <> 1 THEN Bump 1
    IF gTotal(1) <> 14 THEN Bump 1
    IF gValues(1) <> 1 THEN Bump 1

    ' WorkerB: 1 TO 20 -> PRESERVE 1 TO 24
    IF gLo(2) <> 1 THEN Bump 2
    IF gHi(2) <> 24 THEN Bump 2
    IF gDims(2) <> 1 THEN Bump 2
    IF gTotal(2) <> 24 THEN Bump 2
    IF gValues(2) <> 2 THEN Bump 2

    ' Main thread: a third shape, independent of both workers.
    DIM tA(1 TO 3)
    tA(1) = 77
    tA(3) = 99
    IF LBOUND(tA()) <> 1 THEN Bump 1
    IF UBOUND(tA()) <> 3 THEN Bump 1
    IF ARRAYATTR(tA(), 3) <> 1 THEN Bump 1
    IF ARRAYATTR(tA(), 4) <> 3 THEN Bump 1
    IF tA(1) <> 77 THEN Bump 1
    IF tA(3) <> 99 THEN Bump 1

    FOR i = 1 TO 2
        IF gFails(i) <> 0 THEN PRINT "FAIL thread "; i; " count="; gFails(i)
    NEXT i
    PRINT "=== FAILURES: "; gFails(1) + gFails(2)
    FUNCTION = 0
END FUNCTION
