' batch266_cbnm_click.bas - CB.NM* behavioural verification
' Self-triggering: the first callback entry (dialog init) performs
' LISTVIEW SELECT on item 1, which fires a real WM_NOTIFY through the
' modal message loop.  The callback then samples all five CB.NM*
' pseudo-variables and appends one line to cbnm_out.txt.
' Usage: double-click, click "Done" (or the X) to exit.
#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG
    DIALOG NEW 0, "CB.NM click test - click Done to exit", 10, 10, 340, 175 TO hDlg
    CONTROL ADD LISTVIEW, hDlg, 401, "", 10, 10, 310, 120
    LISTVIEW INSERT COLUMN hDlg, 401, 1, "Item", 280, 0
    LISTVIEW INSERT ITEM hDlg, 401, 1, 0, "alpha"
    LISTVIEW INSERT ITEM hDlg, 401, 2, 0, "beta"
    LISTVIEW INSERT ITEM hDlg, 401, 3, 0, "gamma"
    CONTROL ADD BUTTON, hDlg, 200, "Done", 250, 140, 70, 25
    DIALOG SHOW MODAL hDlg CALL DlgCB
    FUNCTION = 0
END FUNCTION

CALLBACK FUNCTION DlgCB() AS LONG
    LOCAL a AS STRING
    LOCAL code, id, hw, nh, sz AS LONG
    STATIC fired AS LONG
    IF fired = 0 THEN
        fired = 1
        LISTVIEW SELECT CB.HNDL, 401, 1
    END IF
    IF CB.MSG = %WM_NOTIFY THEN
        code = CB.NMCODE
        id = CB.NMID
        hw = CB.NMHWND
        nh = CB.NMHDR
        a = CB.NMHDR$
        sz = LEN(a)
        OPEN "cbnm_out.txt" FOR APPEND AS #1
        PRINT #1, "NMCODE=" & STR$(code) & " NMID=" & STR$(id) & " NMHWND=" & STR$(hw) & " NMHDR=" & STR$(nh) & " NMHDR$len=" & STR$(sz)
        CLOSE #1
    END IF
    IF CB.MSG = %WM_COMMAND AND CB.CTL = 200 THEN
        DIALOG END CB.HNDL
    END IF
    FUNCTION = 0
END FUNCTION
