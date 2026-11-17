#COMPILE EXE
#CONSOLE ON
FUNCTION PBMAIN () AS LONG
    LOCAL f AS LONG
    LOCAL i AS LONG
    LOCAL n AS LONG
    LOCAL n1 AS LONG
    LOCAL n2 AS LONG
    LOCAL s1 AS STRING
    LOCAL s4 AS STRING
    LOCAL s6 AS STRING
    LOCAL fails AS LONG
    LOCAL a() AS STRING
    DIM a(1 TO 6)

    Fails = 0
    OPEN "batch379_test.dat" FOR OUTPUT AS #1
    PRINT #1, "L1"
    PRINT #1, "L2"
    PRINT #1, "L3"
    PRINT #1, "L4"
    PRINT #1, "L5"
    CLOSE #1

    ' RECORDS limits the read to three lines; TO reports the actual count.
    OPEN "batch379_test.dat" FOR INPUT AS #1
    LINE INPUT #1, a() RECORDS 3 TO n
    n1 = n
    s1 = a(1)
    s4 = a(4)
    IF n <> 3 THEN INCR Fails
    IF a(1) <> "L1" THEN INCR Fails
    IF a(2) <> "L2" THEN INCR Fails
    IF a(3) <> "L3" THEN INCR Fails
    IF a(4) <> "" THEN INCR Fails
    CLOSE #1

    ' RECORDS larger than the array is capped by the six-element array.
    OPEN "batch379_test.dat" FOR INPUT AS #1
    LINE INPUT #1, a() RECORDS 9 TO n
    n2 = n
    s6 = a(6)
    IF n <> 5 THEN INCR Fails
    IF a(1) <> "L1" THEN INCR Fails
    IF a(2) <> "L2" THEN INCR Fails
    IF a(3) <> "L3" THEN INCR Fails
    IF a(4) <> "L4" THEN INCR Fails
    IF a(5) <> "L5" THEN INCR Fails
    IF a(6) <> "" THEN INCR Fails
    IF EOF(1) = 0 THEN INCR Fails
    CLOSE #1

    ' No RECORDS clause: the array size is the only cap; TO still reports it.
    OPEN "batch379_test.dat" FOR INPUT AS #1
    LINE INPUT #1, a() TO n
    IF n <> 5 THEN INCR Fails
    IF a(1) <> "L1" THEN INCR Fails
    IF a(5) <> "L5" THEN INCR Fails
    IF a(6) <> "" THEN INCR Fails
    CLOSE #1

    OPEN "batch379_test.txt" FOR OUTPUT AS #1
    PRINT #1, "batch379 test, failures="; Fails
    PRINT #1, "n1="; n1; " s1=["; s1; "] s4=["; s4; "] n2="; n2; " s6=["; s6; "]"
    CLOSE #1
    FUNCTION = Fails
END FUNCTION
