' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming CSNG with a string operand, no .exe produced.
FUNCTION PBMAIN () AS LONG
    LOCAL ls AS STRING
    LOCAL n AS LONG
    ls = "abc"
    n = CSNG(ls)
    FUNCTION = 0
END FUNCTION
