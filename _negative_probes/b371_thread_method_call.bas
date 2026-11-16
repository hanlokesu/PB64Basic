' Negative probe - MUST FAIL TO COMPILE.  Not part of the runnable corpus.
' Checked by the skill script check_negative_probes.py.
' Expected: non-zero exit, a diagnostic naming a direct call to the THREAD METHOD, no .exe produced.
CLASS Worker
    INSTANCE ThreadParam AS LONG
    THREAD METHOD MAIN() AS LONG
        METHOD = 1
    END METHOD
    INTERFACE Face
        INHERIT IPOWERTHREAD
    END INTERFACE
END CLASS
FUNCTION PBMAIN () AS LONG
    LOCAL w AS Face
    w = CLASS "Worker"
    w.MAIN()
    FUNCTION = 0
END FUNCTION
