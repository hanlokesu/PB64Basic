#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL h AS QUAD
    LOCAL e AS LONG
    h = CON.HANDLE
    e = ERR
    OPEN "batch378_handle.txt" FOR OUTPUT AS #1
    PRINT #1, "handle="; h
    PRINT #1, "err="; e
    CLOSE #1
    FUNCTION = 0
END FUNCTION
