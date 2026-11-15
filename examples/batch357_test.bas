#COMPILE EXE
#DIM ALL
' batch 357 regression: THREAD CREATE on both targets plus per-thread THREADED
' arrays.  This is the F4 regression probe: the CALLSTK frame table used to be
' process-wide, so four workers overwrote each other's frame and freed the same
' args pointer twice (intermittent 0xC0000374 heap corruption, 9/100 runs).
' It is now thread-local; this example must run clean repeatedly.

THREADED tarr%()
THREADED sarr$()

GLOBAL g_done AS LONG
GLOBAL g_failures AS LONG

THREAD FUNCTION Worker(BYVAL tid AS LONG) AS LONG
    DIM tarr(1 TO 5)
    DIM sarr(1 TO 3)
    LOCAL k AS LONG
    FOR k = 1 TO 5
        tarr(k) = tid * 100 + k
    NEXT k
    sarr(1) = "s" + FORMAT$(tid)
    sarr(2) = "mid"
    sarr(3) = "tail"
    FOR k = 1 TO 5
        IF tarr(k) <> tid * 100 + k THEN g_failures = g_failures + 1
    NEXT k
    IF sarr(1) <> "s" + FORMAT$(tid) THEN g_failures = g_failures + 1
    IF sarr(2) <> "mid" THEN g_failures = g_failures + 1
    IF sarr(3) <> "tail" THEN g_failures = g_failures + 1
    g_done = g_done + 1
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL h(1 TO 4) AS DWORD
    LOCAL i AS LONG
    LOCAL spins AS LONG
    FOR i = 1 TO 4
        THREAD CREATE Worker(i) TO h(i)
        IF h(i) = 0 THEN
            PRINT "THREAD CREATE failed for worker "; i
            g_failures = g_failures + 1
        END IF
    NEXT i
    spins = 0
    DO WHILE g_done < 4 AND spins < 1000
        SLEEP 10
        spins = spins + 1
    LOOP
    FOR i = 1 TO 4
        THREAD CLOSE h(i)
    NEXT i
    IF g_done <> 4 THEN
        PRINT "only "; g_done; " of 4 workers finished"
        g_failures = g_failures + 1
    END IF
    PRINT "workers done="; g_done; " failures="; g_failures
    PRINT "=== FAILURES: "; g_failures
    FUNCTION = 0
END FUNCTION
