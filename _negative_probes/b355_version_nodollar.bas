' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming VERSION without the $ suffix (bare), no .exe produced.
FUNCTION PBMAIN () AS LONG
    LOCAL ver2 AS STRING
    ver2 = VERSION
    FUNCTION = 0
END FUNCTION
