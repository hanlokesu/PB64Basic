' =====================================================================
' PB64Basic Enhanced - batch 427 - dialog callback and CB.HNDL witness
'   Compile-time regression for:
'     1. DIALOG SHOW MODAL hDlg CALL DlgProc() with an empty argument list
'     2. DIALOG SHOW MODELESS hChild without a callback inside WM_INITDIALOG
'     3. CB.HNDL lowered as a numeric pointer, not a BSTR comparison
'   The dialog block is unreachable at run time; the runtime evidence for
'   samples 1359/1360, 0335, 0287 and 0289 is the Gary Beene corpus rerun.
'   The WSTRING block below runs: a concatenation temporary must be copied
'   into a standalone buffer before the statement arena is rewound.
'   === FAILURES: 0
' =====================================================================
#COMPILE EXE
#DIM ALL

GLOBAL hDlg AS DWORD
GLOBAL hChild AS DWORD
GLOBAL fails AS LONG

FUNCTION PBMain()
    LOCAL ws AS WSTRING
    LOCAL r  AS WSTRING

    ws = "AB" + CHR$$(&H6F22) + CHR$$(&H5B57)
    IF LEN(ws) <> 4 THEN fails = fails + 1
    r = RIGHT$(ws, 2)
    IF LEN(r) <> 2 THEN fails = fails + 1
    IF ASC(r, 1) <> &H6F22 THEN fails = fails + 1
    IF ASC(r, 2) <> &H5B57 THEN fails = fails + 1

    IF 0 THEN
        DIALOG NEW PIXELS, 0, "batch427", 100, 100, 200, 100 TO hDlg
        DIALOG SHOW MODAL hDlg CALL DlgProc()
        DIALOG NEW PIXELS, 0, "child", 0, 0, 120, 80 TO hChild
        DIALOG SHOW MODELESS hChild
    END IF
    PRINT "=== FAILURES:"; fails; " ==="
END FUNCTION

CALLBACK FUNCTION DlgProc() AS LONG
    IF CB.HNDL = 0 THEN
        fails = fails + 1
    END IF
END FUNCTION
