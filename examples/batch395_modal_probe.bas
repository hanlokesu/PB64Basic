' batch395_modal_probe.bas - DIALOG SHOW MODAL optional separator commas.
' Both procedures are compiled but never invoked, so run_examples treats this
' file as interactive and only checks that it compiles.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

GLOBAL g_hDialog AS LONG

CALLBACK FUNCTION DlgProc()
    DIALOG END 0, 1
END FUNCTION

SUB ProbeDialogShowModalCallComma()
    DIALOG SHOW MODAL g_hDialog, CALL DlgProc
END SUB

SUB ProbeDialogShowModalToComma()
    LOCAL result AS LONG
    DIALOG SHOW MODAL g_hDialog, TO result
END SUB

FUNCTION PBMAIN() AS LONG
END FUNCTION