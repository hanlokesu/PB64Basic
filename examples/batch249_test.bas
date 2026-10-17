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
    LOCAL hB, hC, hC3, hO, hF, hL, hLn, hT, hS, hLB, hCB, hTab AS LONG
    LOCAL fail AS LONG
    LOCAL cw, ch AS LONG
    LOCAL items() AS STRING
    DIM items(1 TO 3)
    items(1) = "Alpha"
    items(2) = "Beta"
    items(3) = "Gamma"
    DIALOG NEW 0, "batch249 test", 0, 0, 280, 460 TO hDlg
    CONTROL ADD BUTTON, hDlg, 101, "Btn", 10, 10, 60, 14
    CONTROL HANDLE hDlg, 101 TO hB
    CONTROL ADD CHECKBOX, hDlg, 102, "Chk", 10, 30, 60, 14
    CONTROL HANDLE hDlg, 102 TO hC
    CONTROL ADD CHECK3STATE, hDlg, 103, "Chk3", 10, 50, 60, 14
    CONTROL HANDLE hDlg, 103 TO hC3
    CONTROL ADD OPTION, hDlg, 104, "Opt", 10, 70, 60, 14
    CONTROL HANDLE hDlg, 104 TO hO
    CONTROL ADD FRAME, hDlg, 105, "Frame", 10, 90, 120, 60
    CONTROL HANDLE hDlg, 105 TO hF
    CONTROL ADD LABEL, hDlg, 106, "Lbl", 10, 160, 60, 14
    CONTROL HANDLE hDlg, 106 TO hL
    CONTROL ADD LINE, hDlg, 107, "", 10, 180, 100, 4
    CONTROL HANDLE hDlg, 107 TO hLn
    CONTROL ADD TEXTBOX, hDlg, 108, "", 10, 190, 100, 16
    CONTROL HANDLE hDlg, 108 TO hT
    CONTROL ADD SCROLLBAR, hDlg, 109, "", 10, 220, 100, 20
    CONTROL HANDLE hDlg, 109 TO hS
    CONTROL ADD LISTBOX, hDlg, 110, items(), 10, 250, 100, 60
    CONTROL HANDLE hDlg, 110 TO hLB
    CONTROL ADD COMBOBOX, hDlg, 111, items(), 120, 250, 100, 80
    CONTROL HANDLE hDlg, 111 TO hCB
    CONTROL ADD TAB, hDlg, 112, "", 120, 120, 100, 60
    CONTROL HANDLE hDlg, 112 TO hTab
    IF hB = 0 THEN fail = fail + 1
    IF hC = 0 THEN fail = fail + 1
    IF hC3 = 0 THEN fail = fail + 1
    IF hO = 0 THEN fail = fail + 1
    IF hF = 0 THEN fail = fail + 1
    IF hL = 0 THEN fail = fail + 1
    IF hLn = 0 THEN fail = fail + 1
    IF hT = 0 THEN fail = fail + 1
    IF hS = 0 THEN fail = fail + 1
    IF hLB = 0 THEN fail = fail + 1
    IF hCB = 0 THEN fail = fail + 1
    IF hTab = 0 THEN fail = fail + 1
    CONTROL GET SIZE hDlg, 108 TO cw, ch
    IF cw <> 100 OR ch <> 16 THEN
        fail = fail + 1
        ConPrint "FAIL textbox size ->" & STR$(cw) & "x" & STR$(ch)
    END IF
    DIALOG END hDlg
    IF fail = 0 THEN
        ConPrint "batch249 test ok"
    ELSE
        ConPrint "FAILURES=" & STR$(fail)
    END IF
    FUNCTION = 0
END FUNCTION
