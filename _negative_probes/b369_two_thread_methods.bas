' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming a CLASS with two THREAD METHODs, no .exe produced.
CLASS Worker
    INSTANCE ThreadParam AS LONG
    THREAD METHOD ONE() AS LONG
        METHOD = 1
    END METHOD
    THREAD METHOD TWO() AS LONG
        METHOD = 2
    END METHOD
    INTERFACE Face
        INHERIT IPOWERTHREAD
    END INTERFACE
END CLASS
FUNCTION PBMAIN () AS LONG
    LOCAL w AS Face
    w = CLASS "Worker"
    w.LAUNCH(BYVAL 7)
    w.JOIN(w, 1000)
    w.CLOSE()
    FUNCTION = 0
END FUNCTION
