#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL a$, b$, c$, d$, e$ AS STRING
    LOCAL fails AS LONG
    OPEN "batch365_zones.txt" FOR OUTPUT AS #1
    PRINT #1, "A", "B"
    PRINT #1, "1234567890123", "C"
    PRINT #1, "12345678901234", "D"
    PRINT #1, 5, 7
    PRINT #1, "E"; "F"
    CLOSE #1

    OPEN "batch365_zones.txt" FOR INPUT AS #2
    LINE INPUT #2, a$
    LINE INPUT #2, b$
    LINE INPUT #2, c$
    LINE INPUT #2, d$
    LINE INPUT #2, e$
    CLOSE #2

    fails = 0
    IF a$ <> "A" + SPACE$(13) + "B" THEN fails = fails + 1
    IF b$ <> "1234567890123 C" THEN fails = fails + 1
    IF c$ <> "12345678901234" + SPACE$(14) + "D" THEN fails = fails + 1
    IF d$ <> " 5" + SPACE$(13) + "7 " THEN fails = fails + 1
    IF e$ <> "EF" THEN fails = fails + 1
    PRINT "fails="; fails
END FUNCTION
