' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming CINT with a whole array (empty subscripts), no .exe produced.
FUNCTION PBMAIN () AS LONG
    DIM a(1 TO 3) AS LONG
    LOCAL n AS LONG
    n = CINT(a())
    FUNCTION = 0
END FUNCTION
