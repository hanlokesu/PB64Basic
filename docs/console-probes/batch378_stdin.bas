' batch378_stdin - redirected-stdin probe for CON.IN / CON.OUT /
' CON.STDIN.LINE / CON.STDEOF.  Run with stdin piped from two lines.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL s1 AS STRING
    LOCAL s2 AS STRING
    LOCAL e AS LONG

    OPEN "batch378_stdin.txt" FOR OUTPUT AS #1
    PRINT #1, "in="; CON.IN; " out="; CON.OUT

    CON.STDIN.LINE TO s1
    PRINT #1, "line1=[" + s1 + "]"

    CON.STDIN.LINE TO s2
    PRINT #1, "line2=[" + s2 + "]"

    e = CON.STDEOF
    PRINT #1, "eof="; e

    CLOSE #1
END FUNCTION
