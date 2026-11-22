#COMPILE EXE
#DIM ALL

' batch 403: PBWin operator precedence and associativity.
' * and / bind tighter than \, \ binds tighter than MOD, and ^ is
' left-associative.  Unary minus is looser than ^.

FUNCTION PBMAIN () AS LONG
    LOCAL f AS LONG
    LOCAL fails AS LONG
    LOCAL d AS DOUBLE
    LOCAL x AS LONG

    fails = 0
    f = FREEFILE
    OPEN "batch403_precedence_result.txt" FOR OUTPUT AS #f

    IF 30 * 39 \ 60 <> 19 THEN INCR fails
    IF 7 \ 2 * 3 <> 1 THEN INCR fails
    IF 10 MOD 3 * 2 <> 4 THEN INCR fails
    IF 10 * 3 MOD 4 <> 2 THEN INCR fails
    IF 10 \ 3 MOD 2 <> 1 THEN INCR fails
    IF 10 MOD 3 \ 2 <> 0 THEN INCR fails
    IF -7 \ 2 <> -3 THEN INCR fails
    IF -7 MOD 2 <> -1 THEN INCR fails
    IF 2 ^ 3 ^ 2 <> 64 THEN INCR fails
    IF -2 ^ 2 <> -4 THEN INCR fails
    IF 2 ^ -3 <> 0.125 THEN INCR fails

    d = 96.5
    x = d
    IF x <> 96 THEN INCR fails

    PRINT #f, "E1="; 30 * 39 \ 60
    PRINT #f, "E9="; 2 ^ 3 ^ 2
    PRINT #f, "E10="; -2 ^ 2
    PRINT #f, "E11="; 2 ^ -3
    PRINT #f, "ROUND96.5="; x
    PRINT #f, "FAILURES:"; fails
    IF fails = 0 THEN PRINT #f, "PRECEDENCE_PASS"
    CLOSE #f
    FUNCTION = fails
END FUNCTION
