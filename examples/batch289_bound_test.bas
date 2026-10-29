#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL l1 AS LONG, u1 AS LONG, l2 AS LONG, u2 AS LONG
    LOCAL l3 AS LONG, u3 AS LONG, l4 AS LONG, u4 AS LONG
    DIM MyArray(1900 TO 2000, 5 TO 10) AS LONG
    DIM Dyn() AS LONG
    l1 = LBOUND(MyArray)
    u1 = UBOUND(MyArray)
    l2 = LBOUND(MyArray(2))
    u2 = UBOUND(MyArray, 2)
    l3 = LBOUND(MyArray(1))
    u3 = UBOUND(MyArray(1))
    l4 = LBOUND(Dyn())
    u4 = UBOUND(Dyn())
    OPEN "b289b_out.txt" FOR OUTPUT AS #1
    PRINT #1, "l1="; l1; "u1="; u1
    PRINT #1, "l2="; l2; "u2="; u2
    PRINT #1, "l3="; l3; "u3="; u3
    PRINT #1, "l4="; l4; "u4="; u4
    CLOSE #1
END FUNCTION
