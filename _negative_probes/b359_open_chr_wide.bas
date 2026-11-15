' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming OPEN CHR=WIDE (not implemented), no .exe produced.
FUNCTION PBMAIN () AS LONG
    OPEN "b359_wide.txt" FOR OUTPUT AS #1 CHR=WIDE
    PRINT #1, "x"
    CLOSE #1
    FUNCTION = 0
END FUNCTION
