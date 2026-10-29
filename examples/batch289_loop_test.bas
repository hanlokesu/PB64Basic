#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL i AS LONG
    OPEN "b289l_out.txt" FOR OUTPUT AS #1
    i = 0
    WHILE i < 100
        INCR i
        IF i = 5 THEN EXIT LOOP
    WEND
    PRINT #1, "w1="; i
    i = 0
    DO WHILE i > 5
        INCR i
    LOOP
    PRINT #1, "d1="; i
    i = 0
    DO UNTIL i >= 3
        INCR i
    LOOP
    PRINT #1, "d2="; i
    i = 0
    DO
        INCR i
    LOOP UNTIL i >= 4
    PRINT #1, "d3="; i
    i = 0
    DO
        INCR i
    LOOP WHILE i < 2
    PRINT #1, "d4="; i
    i = 0
    WHILE -1
        INCR i
        IF i = 3 THEN EXIT LOOP
    WEND
    PRINT #1, "w2="; i
    CLOSE #1
END FUNCTION
