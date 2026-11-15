' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming OPEN sequential INPUT CHR=WIDE before the AS clause, no .exe produced.
FUNCTION PBMAIN () AS LONG
    OPEN "b360_wide_in.txt" FOR INPUT CHR=WIDE AS #1
    CLOSE #1
    FUNCTION = 0
END FUNCTION
