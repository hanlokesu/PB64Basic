' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming TCP NOTIFY with an event list but no TO hWnd AS wMsg, no .exe produced.
FUNCTION PBMAIN () AS LONG
    LOCAL h AS LONG
    TCP NOTIFY #1, ACCEPT
    FUNCTION = 0
END FUNCTION
