' batch258d_test.bas - CALLSTK$ parameter snapshot + CALLSTK statement
#COMPILE EXE
SUB Callee(x AS LONG, y AS STRING)
    LOCAL h, ok AS LONG
    LOCAL s1, s2, s9 AS STRING
    ok = 1
    h = FREEFILE
    OPEN "batch258d_out.txt" FOR OUTPUT AS #h
    s1 = CALLSTK$(1)
    s2 = CALLSTK$(2)
    s9 = CALLSTK$(9)
    IF s1 <> "CALLEE(7," & CHR$(34) & "hello" & CHR$(34) & ")" THEN ok = 0
    IF s2 <> "PBMAIN()" THEN ok = 0
    IF s9 <> "" THEN ok = 0
    CALLSTK "batch258d_dump.txt"
    IF ok THEN PRINT #h, "batch258d test ok" ELSE PRINT #h, "batch258d test FAIL"
    CLOSE #h
END SUB

FUNCTION PBMAIN() AS LONG
    CALL Callee(7, "hello")
    FUNCTION = 0
END FUNCTION
