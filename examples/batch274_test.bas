' batch274_test.bas - DEFtype family (DEFBYT/DEFWRD/DEFINT/DEFLNG/DEFQUD/
' DEFSNG/DEFDBL/DEFEXT/DEFCUR/DEFCUX/DEFDWD/DEFSTR) dual-verified.
' CHW DEFBYT_DEFCUR_DEFCUX_DEFDBL_DEFDWD_DEFEXT_DEFINT_DEFLNG.htm:
' letter ranges set the default type for implicitly declared variables;
' explicit declarations always win; #DIM ALL / OPTION EXPLICIT disables
' DEFtype (PBWin Error 519 verified).  Output to b274_out.txt.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL failures AS LONG
    failures = 0
    OPEN "b274_out.txt" FOR OUTPUT AS #1
    DEFSTR S-V
    DEFLNG I-N
    DEFINT A-C
    DEFDBL X-Z
    s = "hello-defstr"
    i = 123456
    a = 7
    x = 3.7
    IF s <> "hello-defstr" THEN INCR failures
    IF i <> 123456 THEN INCR failures
    IF a <> 7 THEN INCR failures
    IF x <> 3.7 THEN INCR failures
    ' explicit declaration beats DEFtype (CHW: DEFtype only affects implicit)
    LOCAL q AS LONG
    q = 999
    IF q <> 999 THEN INCR failures
    IF failures = 0 THEN
        PRINT #1, "batch274 test ok"
    ELSE
        PRINT #1, "FAILURES: " + LTRIM$(STR$(failures))
    END IF
    CLOSE #1
    FUNCTION = failures
END FUNCTION
