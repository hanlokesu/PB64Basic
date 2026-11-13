' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming BIN$ with a string operand, no .exe produced.
FUNCTION PBMAIN () AS LONG
    LOCAL ls AS STRING
    LOCAL x AS STRING
    ls = "abc"
    x = BIN$(ls)
    FUNCTION = 0
END FUNCTION
