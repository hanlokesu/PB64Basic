' batch244_test.bas - CONTROL ADD default styles (batch 244 alignment)
' (rebuilt from the batch's real optimization topic; see SKILL.md)
#COMPILE EXE
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
    LOCAL hB, hC, hT, hO AS LONG
    LOCAL fail AS LONG
    DIALOG NEW 0, "batch244 test", 0, 0, 200, 220 TO hDlg
    CONTROL ADD BUTTON, hDlg, 101, "Btn", 10, 10, 60, 14
    CONTROL ADD CHECKBOX, hDlg, 102, "Chk", 10, 30, 60, 14
    CONTROL ADD TEXTBOX, hDlg, 103, "txt", 10, 50, 120, 20
    CONTROL ADD OPTION, hDlg, 104, "Opt", 10, 80, 60, 14
    CONTROL HANDLE hDlg, 101 TO hB
    CONTROL HANDLE hDlg, 102 TO hC
    CONTROL HANDLE hDlg, 103 TO hT
    CONTROL HANDLE hDlg, 104 TO hO
    IF hB = 0 THEN fail = fail + 1
    IF hC = 0 THEN fail = fail + 1
    IF hT = 0 THEN fail = fail + 1
    IF hO = 0 THEN fail = fail + 1
    IF fail = 0 THEN
        ConPrint "batch244 test ok"
    ELSE
        ConPrint "batch244 FAILURES: " & STR$(fail)
    END IF
    DIALOG END hDlg
    FUNCTION = 0
END FUNCTION

