' batch362_test.bas - REDIM PRESERVE shrink-then-regrow tail clearing
' and PRINTERCOUNT dynamic EnumPrintersW smoke.  The REDIM expectations
' (101 202 0 0) are PBWin10-controlled: shrink clears the dropped slots.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL a() AS LONG
    LOCAL s() AS STRING
    LOCAL pc AS LONG
    LOCAL fails AS LONG
    REDIM a(1 TO 4)
    a(1) = 101
    a(2) = 202
    a(3) = 303
    a(4) = 404
    REDIM PRESERVE a(1 TO 2)
    REDIM PRESERVE a(1 TO 4)
    IF a(1) <> 101 THEN fails = fails + 1
    IF a(2) <> 202 THEN fails = fails + 1
    IF a(3) <> 0 THEN fails = fails + 1
    IF a(4) <> 0 THEN fails = fails + 1
    REDIM s(1 TO 3)
    s(1) = "a"
    s(2) = "b"
    s(3) = "c"
    REDIM PRESERVE s(1 TO 1)
    REDIM PRESERVE s(1 TO 3)
    IF s(1) <> "a" THEN fails = fails + 1
    IF LEN(s(2)) <> 0 THEN fails = fails + 1
    IF LEN(s(3)) <> 0 THEN fails = fails + 1
    pc = PRINTERCOUNT
    IF pc < 0 THEN fails = fails + 1
    OPEN "batch362_out.txt" FOR OUTPUT AS #1
    PRINT #1, "a=" & STR$(a(1)) & "," & STR$(a(2)) & "," & STR$(a(3)) & "," & STR$(a(4))
    PRINT #1, "s=" & s(1) & "," & s(2) & "," & s(3)
    PRINT #1, "printers=" & STR$(pc)
    CLOSE #1
    IF fails <> 0 THEN FUNCTION = 1 ELSE FUNCTION = 0
END FUNCTION