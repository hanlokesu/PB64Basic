#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN () AS LONG
    LOCAL i AS LONG
    LOCAL fails AS LONG
    DIM a(1 TO 4) AS LONG
    fails = 0
    OPEN "batch378_redim_stages.txt" FOR OUTPUT AS #1

    a(1) = 11
    a(2) = 22
    a(3) = 33
    a(4) = 44

    ARRAY REDIM INCR a(), 4
    PRINT #1, "g1 ub="; UBOUND(a); " lb="; LBOUND(a); " a1..8="; a(1); a(2); a(3); a(4); a(5); a(6); a(7); a(8); " fails="; fails
    IF UBOUND(a) <> 8 THEN INCR fails
    IF LBOUND(a) <> 1 THEN INCR fails
    IF a(1) <> 11 OR a(2) <> 22 OR a(3) <> 33 OR a(4) <> 44 THEN INCR fails
    FOR i = 5 TO 8
        IF a(i) <> 0 THEN INCR fails
    NEXT i

    ARRAY REDIM DECR a(), 2
    PRINT #1, "g2 ub="; UBOUND(a); " lb="; LBOUND(a); " a1..6="; a(1); a(2); a(3); a(4); a(5); a(6); " fails="; fails
    IF UBOUND(a) <> 6 THEN INCR fails
    IF a(1) <> 11 OR a(2) <> 22 OR a(3) <> 33 OR a(4) <> 44 THEN INCR fails
    FOR i = 5 TO 6
        IF a(i) <> 0 THEN INCR fails
    NEXT i

    ' Grow past the first realloc capacity again; the prefix must survive.
    ARRAY REDIM INCR a(), 4
    PRINT #1, "g3 ub="; UBOUND(a); " lb="; LBOUND(a); " a1..10="; a(1); a(2); a(3); a(4); a(5); a(6); a(7); a(8); a(9); a(10); " fails="; fails
    IF UBOUND(a) <> 10 THEN INCR fails
    IF a(1) <> 11 OR a(2) <> 22 OR a(3) <> 33 OR a(4) <> 44 THEN INCR fails
    FOR i = 5 TO 10
        IF a(i) <> 0 THEN INCR fails
    NEXT i

    ' Shrink to one element, then grow again: only a(1) survives.
    ARRAY REDIM DECR a(), 9
    PRINT #1, "g4 ub="; UBOUND(a); " lb="; LBOUND(a); " a1="; a(1); " fails="; fails
    IF UBOUND(a) <> 1 THEN INCR fails
    IF a(1) <> 11 THEN INCR fails
    ARRAY REDIM INCR a(), 2
    PRINT #1, "g5 ub="; UBOUND(a); " lb="; LBOUND(a); " a1..3="; a(1); a(2); a(3); " fails="; fails
    IF UBOUND(a) <> 3 THEN INCR fails
    IF a(1) <> 11 OR a(2) <> 0 OR a(3) <> 0 THEN INCR fails

    PRINT #1, "batch378 redim, failures="; fails
    CLOSE #1
    PRINT "batch378 redim, failures="; fails
    OPEN "batch378_redim_result.txt" FOR OUTPUT AS #1
    PRINT #1, "batch378 redim, failures="; fails
    CLOSE #1
END FUNCTION
