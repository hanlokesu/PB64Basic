#COMPILE EXE

'=====================================================================
' PB64Basic Enhanced - batch 412 regression test
'---------------------------------------------------------------------
' Purpose: lock the LISTVIEW default extended-style split that batch 412
'          restored while keeping sample 0294 pixel-identical.
'
'           Rule under test:
'             * style operand omitted  -> documented DDT default, which
'               includes %LVS_EX_FULLROWSELECT (&H20).
'             * explicit style supplied (0294 passes only %WS_BORDER)
'               -> no implicit %LVS_EX_FULLROWSELECT; the caller's
'               extended style starts at zero.
'
'           The two witnesses are the same LISTVIEW GET STYLEXX statement
'           on two controls created in one dialog, so a regression that
'           either drops the default or re-adds it to explicit styles
'           fails one of the two assertions.
'
' Expected output:
'   default style xx -> &H20
'   explicit style xx -> &H0
'   === FAILURES: 0 ===
'   exit code 0
'=====================================================================

DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL hDlg AS LONG
    LOCAL hLv  AS LONG
    LOCAL n    AS LONG
    LOCAL fail AS LONG

    DIALOG NEW 0, "batch412 LISTVIEW default extended style", 0, 0, 320, 180 TO hDlg
    DIALOG SHOW MODELESS hDlg

    ' Control 401: style operand omitted -> documented DDT default.
    CONTROL ADD LISTVIEW, hDlg, 401, "", 10, 10, 300, 70
    CONTROL HANDLE hDlg, 401 TO hLv
    IF hLv = 0 THEN fail = fail + 1

    n = -1
    LISTVIEW GET STYLEXX hDlg, 401 TO n
    ConPrint "default style xx -> &H" & HEX$(n)
    IF n <> &H20 THEN fail = fail + 1

    ' Control 402: explicit style with no LVS_TYPEMASK bits (the 0294
    ' shape).  It still becomes a report view, but its extended style must
    ' stay exactly what the caller supplied: zero.
    CONTROL ADD LISTVIEW, hDlg, 402, "", 10, 90, 300, 70, &H800000
    n = -1
    LISTVIEW GET STYLEXX hDlg, 402 TO n
    ConPrint "explicit style xx -> &H" & HEX$(n)
    IF n <> 0 THEN fail = fail + 1

    DIALOG END hDlg, fail
    ConPrint ""
    ConPrint "=== FAILURES:" & STR$(fail) & "==="
    FUNCTION = fail
END FUNCTION
