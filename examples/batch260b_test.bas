' batch260b_test.bas - COMM family error-code dual assertion (batch260)
' CHW COMM_* pages: queries on an unopened channel set ERR 5; OPEN of a
' non-existent port sets ERR 69; all other statements on an unopened
' channel set ERR 52.  PBWin probe values verified on both sides.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL e, n, r, h AS LONG
    LOCAL s, s2 AS STRING
    LOCAL ok AS LONG
    ok = 1
    s = ""
    h = FREEFILE
    OPEN "batch260b_out.txt" FOR OUTPUT AS #h
    r = COMM(1, TXQUE)
    e = ERR
    IF e <> 5 THEN ok = 0
    s = "get " & STR$(e) & " " & STR$(r)
    PRINT #h, s
    COMM OPEN "COM404" AS #2
    e = ERR
    IF e <> 69 THEN ok = 0
    s = "open " & STR$(e)
    PRINT #h, s
    COMM SEND #3, "abc" TO n
    e = ERR
    IF e <> 52 THEN ok = 0
    s = "send " & STR$(e) & " " & STR$(n)
    PRINT #h, s
    COMM PRINT #4, "abc"
    e = ERR
    IF e <> 52 THEN ok = 0
    s = "print " & STR$(e)
    PRINT #h, s
    COMM RECV #5, 10, s2
    e = ERR
    IF e <> 52 THEN ok = 0
    s = "recv " & STR$(e)
    PRINT #h, s
    COMM LINE #6, s2
    e = ERR
    IF e <> 52 THEN ok = 0
    s = "line " & STR$(e)
    PRINT #h, s
    COMM SET #7, BAUD = 9600
    e = ERR
    IF e <> 52 THEN ok = 0
    s = "set " & STR$(e)
    PRINT #h, s
    COMM RESET #8, FLOW
    e = ERR
    IF e <> 52 THEN ok = 0
    s = "reset " & STR$(e)
    PRINT #h, s
    COMM TIMEOUT #9, 1000
    e = ERR
    IF e <> 52 THEN ok = 0
    s = "timeout " & STR$(e)
    PRINT #h, s
    COMM CLOSE #10
    e = ERR
    IF e <> 52 THEN ok = 0
    s = "close " & STR$(e)
    PRINT #h, s
    IF ok THEN
        s = "batch260b test ok"
    ELSE
        s = "batch260b test FAIL"
    END IF
    PRINT #h, s
    CLOSE #h
    FUNCTION = 0
END FUNCTION
