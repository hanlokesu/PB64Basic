#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL r AS LONG
    r = 5
    OPEN "b284semi.txt" FOR OUTPUT AS #1
    PRINT #1, "A"; "B"
    PRINT #1, "A "; "B"
    PRINT #1, "A"; r
    PRINT #1, "A "; r
    PRINT #1, "A" ; "B" ; r
    CLOSE #1
END FUNCTION
