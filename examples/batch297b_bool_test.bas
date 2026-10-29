' batch297b_bool_test.bas - bool print representation check
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    OPEN "batch297b_out.txt" FOR OUTPUT AS #1
    PRINT #1, "a="; (1 <> 0)
    PRINT #1, "b="; (2 = 2)
    PRINT #1, "c="; (0 = 1)
    CLOSE #1
    FUNCTION = 0
END FUNCTION
