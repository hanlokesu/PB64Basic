' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming THREAD OBJECT (IPowerThread), no .exe produced.
FUNCTION PBMAIN () AS LONG
    LOCAL h AS LONG
    THREAD OBJECT h
    FUNCTION = 0
END FUNCTION
