#COMPILE EXE
#DIM ALL
' batch275_test.bas - D-batch: READ$/DATA unquoted-item case + DATACOUNT
' CHW READ_function.htm / DATA_statement.htm.  Dual-checked against PBWin10:
' READ$(n) 1-based random access; unquoted DATA items trimmed but keep case;
' DATACOUNT = total item count across all DATA lines.

FUNCTION doubleIt(BYVAL x AS LONG) AS LONG
    FUNCTION = x * 2
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL i AS LONG
    LOCAL n AS LONG
    LOCAL d AS QUAD
    LOCAL failures AS LONG

    OPEN "b275_out.txt" FOR OUTPUT AS #1

    DATA 10, 20, "hi", 30
    DATA "  spaced  ", plain

    n = DATACOUNT
    IF n <> 6 THEN INCR failures
    s = READ$(1)
    IF s <> "10" THEN INCR failures
    s = READ$(3)
    IF s <> "hi" THEN INCR failures
    s = READ$(4)
    IF s <> "30" THEN INCR failures
    IF VAL(READ$(1)) <> 10 THEN INCR failures
    ' quoted item keeps spaces, unquoted item trimmed but case preserved
    s = READ$(5)
    IF s <> "  spaced  " THEN INCR failures
    s = READ$(6)
    IF s <> "plain" THEN INCR failures
    ' out-of-range READ$ returns "" with no error
    IF READ$(99) <> "" THEN INCR failures

    ' DECR / DO..LOOP / DECLARE + user function / DIR$ / DISKFREE / DISKSIZE
    i = 5
    DECR i
    IF i <> 4 THEN INCR failures
    i = 0
    DO WHILE i < 3
        INCR i
    LOOP
    IF i <> 3 THEN INCR failures
    DO UNTIL i >= 5
        INCR i
    LOOP
    IF i <> 5 THEN INCR failures
    i = 0
    DO
        INCR i
        IF i = 2 THEN EXIT DO
    LOOP
    IF i <> 2 THEN INCR failures
    IF doubleIt(21) <> 42 THEN INCR failures
    s = DIR$(ENVIRON$("TEMP") + "\*.*")
    IF s = "" THEN INCR failures
    s = DIR$(NEXT)
    DIR$ CLOSE
    d = DISKFREE("C:\")
    IF d <= 0 THEN INCR failures
    d = DISKSIZE("C:\")
    IF d <= 0 THEN INCR failures

    IF failures = 0 THEN
        PRINT #1, "batch275 test ok"
    ELSE
        PRINT #1, "batch275 test FAILURES: " + LTRIM$(STR$(failures))
    END IF
    CLOSE #1
    FUNCTION = 0
END FUNCTION
