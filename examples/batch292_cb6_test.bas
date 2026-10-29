' batch292_cb6_test.bas - CB.* six callback pseudo-variables (CBCTL/CBCTLMSG/
' CBHNDL/CBLPARAM/CBMSG/CBWPARAM; CHW CB_Callback_functions.htm).  The callback
' is declared but never fired (no DIALOG SHOW - a modal loop would hang
' headless), so the lines prove both compilers accept the official syntax and
' link the runtime symbols.
#COMPILE EXE
GLOBAL hDlg AS LONG

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "CB6", 10, 10, 200, 100 TO hDlg
    DIALOG END hDlg
    OPEN "batch292_out.txt" FOR OUTPUT AS #1
    PRINT #1, "batch292 test ok"
    CLOSE #1
    FUNCTION = 0
END FUNCTION

CALLBACK FUNCTION DlgCB() AS LONG
    LOCAL ctl, ctlmsg, hndl, lparam, msg, wparam AS LONG
    ctl = CB.CTL
    ctlmsg = CB.CTLMSG
    hndl = CB.HNDL
    lparam = CB.LPARAM
    msg = CB.MSG
    wparam = CB.WPARAM
    OPEN "batch292_out.txt" FOR APPEND AS #1
    PRINT #1, "cb6:" + STR$(ctl) + ":" + STR$(ctlmsg) + ":" + STR$(hndl) + ":" + STR$(lparam) + ":" + STR$(msg) + ":" + STR$(wparam)
    CLOSE #1
    FUNCTION = 0
END FUNCTION
