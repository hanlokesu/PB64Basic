' batch258c_test.bas - CHOOSE (ELSE/BIT/BITS + lazy) / CODEPTR
' COMMAND$ quote-parsing is covered by the dual-run probe (_b258c_probe.bas),
' which is launched with command-line arguments; the headless harness runs
' every example with no arguments, so COMMAND$ asserts are not meaningful here.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL h, ok AS LONG
    LOCAL c1, c2, c3, c4 AS LONG
    LOCAL cb AS STRING
    LOCAL p AS LONG
    ok = 1
    h = FREEFILE
    OPEN "batch258c_out.txt" FOR OUTPUT AS #h
    c1 = CHOOSE(2, 10, 20, 30)
    IF c1 <> 20 THEN ok = 0
    c2 = CHOOSE(4, 10, 20, 30)
    IF c2 <> 0 THEN ok = 0
    c3 = CHOOSE(4, 10, 20, 30 ELSE 99)
    IF c3 <> 99 THEN ok = 0
    c4 = CHOOSE(BIT 5, 10, 20, 30)
    IF c4 <> 10 THEN ok = 0
    cb = CHOOSE$(BITS 5, "a", "b", "c")
    IF cb <> "ac" THEN ok = 0
    p = CODEPTR(MySub)
    IF p = 0 THEN ok = 0
    IF ok THEN PRINT #h, "batch258c test ok" ELSE PRINT #h, "batch258c test FAIL"
    CLOSE #h
    IF ok THEN FUNCTION = 0 ELSE FUNCTION = 1
END FUNCTION

SUB MySub()
END SUB
