' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming CINT with a whole structure operand, no .exe produced.
TYPE T348
    x AS LONG
    y AS LONG
END TYPE
FUNCTION PBMAIN () AS LONG
    DIM t AS T348
    LOCAL n AS LONG
    n = CINT(t)
    FUNCTION = 0
END FUNCTION
