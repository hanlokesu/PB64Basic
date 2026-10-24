' batch271_test.bas - NEWCOM real COM instantiation (CHW Creating_a_DISPATCH_
' Object / Object_Data_Types).  PBWin-official shape: `LET o = NEWCOM "ProgID"`
' on an IDISPATCH object variable, verified with ISOBJECT(); OBJRESULT stays 0
' on both the success and the failure path (PBWin-measured).  Uses the system
' COM component Shell.Application.  Output goes to b271_out.txt.
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL o AS IDISPATCH
    LOCAL f AS IDISPATCH
    LOCAL ok AS LONG
    LOCAL r AS LONG
    LOCAL failures AS LONG
    failures = 0

    LET o = NEWCOM "Shell.Application"
    IF ISOBJECT(o) THEN ok = 1
    r = OBJRESULT
    IF ok <> 1 THEN INCR failures
    IF r <> 0 THEN INCR failures

    LET f = NEWCOM "NoSuchProgID.XYZ"
    IF ISOBJECT(f) THEN INCR failures
    r = OBJRESULT
    IF r <> 0 THEN INCR failures

    OPEN "b271_out.txt" FOR OUTPUT AS #1
    IF failures = 0 THEN
        PRINT #1, "batch271 test ok"
    ELSE
        PRINT #1, "FAILURES: " + LTRIM$(STR$(failures))
    END IF
    CLOSE #1
    FUNCTION = failures
END FUNCTION
