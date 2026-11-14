' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming ASC statement on a WSTRING left-hand variable, no .exe produced.
FUNCTION PBMAIN () AS LONG
    LOCAL ws AS WSTRING
    ASC(ws, 1) = 72
    FUNCTION = 0
END FUNCTION
