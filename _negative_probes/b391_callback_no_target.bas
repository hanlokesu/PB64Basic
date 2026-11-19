' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming CALLBACK without a FUNCTION or SUB target, no .exe produced.
CALLBACK
FUNCTION PBMAIN () AS LONG
    FUNCTION = 0
END FUNCTION
