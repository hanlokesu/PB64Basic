' batch384_stdin_edge - redirected stdin long-line and Ctrl-Z EOF probe.
' Long input: 5000 X, then "tail".  Ctrl-Z input: "A" + CHR$(26) + "B".
' Run with the matching input file on stdin.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL s1 AS STRING
    LOCAL s2 AS STRING
    LOCAL len1 AS LONG
    LOCAL len2 AS LONG
    LOCAL e1 AS LONG
    LOCAL e2 AS LONG
    LOCAL fails AS LONG

    OPEN "batch384_stdin_edge.txt" FOR OUTPUT AS #1

    CON.STDIN.LINE TO s1
    len1 = LEN(s1)
    e1 = CON.STDEOF
    CON.STDIN.LINE TO s2
    len2 = LEN(s2)
    e2 = CON.STDEOF

    PRINT #1, "len1="; len1; " head=["; LEFT$(s1, 8); "] tail=["; RIGHT$(s1, 8); "] eof1="; e1
    PRINT #1, "len2="; len2; " line2=["; s2; "] eof2="; e2

    fails = 0
    IF len1 = 5000 THEN
        IF e1 <> 0 THEN fails = fails + 1
        IF len2 <> 4 THEN fails = fails + 1
        IF e2 <> 1 THEN fails = fails + 1
    END IF
    IF len1 = 1 THEN
        IF e1 <> 1 THEN fails = fails + 1
        IF len2 <> 0 THEN fails = fails + 1
        IF e2 <> 1 THEN fails = fails + 1
    END IF
    IF len1 <> 5000 AND len1 <> 1 THEN fails = fails + 1

    PRINT #1, "failures="; fails
    CLOSE #1
    FUNCTION = fails
END FUNCTION
