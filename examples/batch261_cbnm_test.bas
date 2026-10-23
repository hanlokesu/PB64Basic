' batch261_cbnm_test.bas - CB.NM* five pseudo-variables + CALLBACK FUNCTION
' (CHW CB_Callback_functions.htm / callbacks.htm).  The callback is declared
' but never fired (no DIALOG SHOW - a modal loop would hang headless), so the
' CB.NM* lines prove both compilers accept the official syntax and link the
' runtime symbols; behavioural verification needs a GUI message loop.
#COMPILE EXE
GLOBAL hDlg AS LONG

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "CB", 10, 10, 200, 100 TO hDlg
    DIALOG END hDlg
    OPEN "batch261_out.txt" FOR OUTPUT AS #1
    PRINT #1, "batch261 test ok"
    CLOSE #1
    FUNCTION = 0
END FUNCTION

CALLBACK FUNCTION DlgCB() AS LONG
    LOCAL a, s AS STRING
    LOCAL code, id, hw, nh, sz AS LONG
    code = CB.NMCODE
    id = CB.NMID
    hw = CB.NMHWND
    nh = CB.NMHDR
    a = CB.NMHDR$
    sz = LEN(a)
    s = "cb:" + STR$(code) + ":" + STR$(id) + ":" + STR$(hw) + ":" + STR$(nh) + ":" + STR$(sz)
    OPEN "batch261_out.txt" FOR APPEND AS #1
    PRINT #1, s
    CLOSE #1
    FUNCTION = 0
END FUNCTION
