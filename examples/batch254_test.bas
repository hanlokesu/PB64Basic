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
    LOCAL hDlg AS LONG
    LOCAL hTab, hSc, hBtn AS LONG
    LOCAL fail AS LONG
    LOCAL c AS LONG
    LOCAL tx AS STRING
    DIALOG NEW 0, "batch254 test", 0, 0, 200, 160 TO hDlg
    CONTROL ADD TAB, hDlg, 101, "Tab1", 10, 10, 120, 60
    CONTROL HANDLE hDlg, 101 TO hTab
    CONTROL ADD SCROLLBAR, hDlg, 102, "hello", 10, 80, 100, 20
    CONTROL HANDLE hDlg, 102 TO hSc
    CONTROL ADD BUTTON, hDlg, 103, "Btn", 10, 110, 60, 14
    CONTROL HANDLE hDlg, 103 TO hBtn
    IF hTab = 0 THEN fail = fail + 1
    IF hSc = 0 THEN fail = fail + 1
    IF hBtn = 0 THEN fail = fail + 1
    ' TAB() official function forms (CHW TAB_statement.htm)
    c = TAB(COUNT, hDlg, 101)
    IF c < 0 THEN fail = fail + 1
    c = TAB(SELECT, hDlg, 101)
    IF c < 0 THEN fail = fail + 1
    tx = TAB$(TEXT, hDlg, 101, 1)
    ' CONTROL by-id verbs (batch 251; CHW has no CONTROL SHOW - omitted)
    CONTROL ENABLE hDlg, 103
    CONTROL SET TEXT hDlg, 103, "Btn2"
    CONTROL GET TEXT hDlg, 103 TO tx
    IF tx <> "Btn2" THEN fail = fail + 1
    CONTROL SET CHECK hDlg, 103, 0
    CONTROL GET CHECK hDlg, 103 TO c
    DIALOG END hDlg
    IF fail = 0 THEN
        ConPrint "batch254 test ok"
    ELSE
        ConPrint "FAILURES=" & STR$(fail)
    END IF
    FUNCTION = 0
END FUNCTION
