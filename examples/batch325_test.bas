#COMPILE EXE

' batch 325 self-check: THREADED arrays (CHW THREADED statement).
' Each worker thread dimensions the same two THREADED arrays (one numeric,
' one string), writes a thread-specific pattern, and reads it back from a
' separate helper Sub to prove cross-procedure visibility. Every thread must
' observe only its own copy; the main thread keeps an independent copy that
' the workers must never touch.
' The string array is dimensioned with a bare DIM (no AS clause); the string
' type is inherited from the THREADED declaration. An explicit "AS STRING"
' DIM triggers a code-generation fault in the reference compiler.

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

THREADED tarr%()
THREADED sarr() AS STRING

GLOBAL g_done AS LONG
GLOBAL g_failures AS LONG

' Separate procedure: reads the calling thread's THREADED arrays.
SUB CheckCopy(BYVAL tid AS LONG)
    DIM k AS LONG
    FOR k = 1 TO 5
        IF tarr(k) <> tid * 100 + k THEN
            g_failures = g_failures + 1
        END IF
    NEXT k
    IF sarr(1) <> "s" + FORMAT$(tid) THEN
        g_failures = g_failures + 1
    END IF
    IF sarr(3) <> FORMAT$(tid * 3) THEN
        g_failures = g_failures + 1
    END IF
END SUB

THREAD FUNCTION Worker(BYVAL tid AS LONG) AS LONG
    DIM tarr(1 TO 5)
    DIM sarr(1 TO 3)
    DIM k AS LONG
    FOR k = 1 TO 5
        tarr(k) = tid * 100 + k
    NEXT k
    sarr(1) = "s" + FORMAT$(tid)
    sarr(2) = "mid"
    sarr(3) = FORMAT$(tid * 3)
    CheckCopy tid
    g_done = g_done + 1
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL N AS LONG, i AS LONG, waits AS LONG, fails AS LONG, rc AS LONG
    DIM hh(1 TO 8) AS DWORD
    N = 4
    DIM tarr(1 TO 5)
    DIM sarr(1 TO 3)
    FOR i = 1 TO 5
        tarr(i) = 900 + i
    NEXT i
    sarr(1) = "main"
    FOR i = 1 TO N
        THREAD CREATE Worker(i) TO hh(i)
    NEXT i
    waits = 0
    DO WHILE g_done < N AND waits < 500
        SLEEP 20
        waits = waits + 1
    LOOP
    FOR i = 1 TO N
        THREAD CLOSE hh(i) TO rc
    NEXT i
    fails = g_failures
    FOR i = 1 TO 5
        IF tarr(i) <> 900 + i THEN
            fails = fails + 1
        END IF
    NEXT i
    IF sarr(1) <> "main" THEN
        fails = fails + 1
    END IF
    ConPrint "done=" + FORMAT$(g_done) + " expected=" + FORMAT$(N)
    ConPrint "main_t1=" + FORMAT$(tarr(1)) + " expected=901"
    ConPrint "main_s1=" + sarr(1)
    ConPrint "=== FAILURES: " + FORMAT$(fails)
    FUNCTION = 0
END FUNCTION
