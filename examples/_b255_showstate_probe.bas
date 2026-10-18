' _b255_showstate_probe.bas - CONTROL SHOW STATE dual-run probe (CHW)
' Syntax: CONTROL SHOW STATE hDlg, id&, showstate& [TO lResult&]
' showstate 0-10; TO returns 0 if previously not visible, nonzero if visible.
' ConPrint template from batch177_test.bas (verified MATCH).
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
FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG, r1 AS LONG, r2 AS LONG, r3 AS LONG
    DIALOG NEW 0, "showstate probe", 0, 0, 200, 100 TO hDlg
    DIALOG SHOW MODELESS hDlg        ' displayed dialog: PBWin TO = real visibility
    CONTROL ADD BUTTON, hDlg, 101, "OK", 10, 10, 40, 14
    ' %SW_HIDE=0 %SW_MAXIMIZE=1 %SW_MINIMIZE=2 %SW_RESTORE=3 %SW_SHOW=5
    ' r1: hide while visible -> previously visible (nonzero)
    CONTROL SHOW STATE hDlg, 101, 0 TO r1        ' %SW_HIDE
    ' r2: show after hide -> previously hidden (zero)
    CONTROL SHOW STATE hDlg, 101, 5 TO r2        ' %SW_SHOW
    ' r3: show while visible -> previously visible (nonzero)
    CONTROL SHOW STATE hDlg, 101, 5 TO r3        ' %SW_SHOW
    ConPrint "r1=" & STR$(r1) & " r2=" & STR$(r2) & " r3=" & STR$(r3)
    ' batch 255 user decision: fork TO = -3 (PBWin-aligned)
    IF r1 = -3 AND r2 = -3 AND r3 = -3 THEN
        ConPrint "SHOW STATE TO = -3 (PBWin-aligned): ok"
    ELSE
        ConPrint "SHOW STATE TO semantics: WRONG"
    END IF
    DIALOG END hDlg
END FUNCTION
