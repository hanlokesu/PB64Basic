' batch251_test.bas - CONTROL verbs by-id (batch 251: ENABLE/DISABLE/HIDE/SET TEXT/GET CHECK/SET CHECK/KILL)
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
    LOCAL hBtn AS LONG
    LOCAL chk AS LONG
    LOCAL fail AS LONG
    DIALOG NEW 0, "batch251 test", 0, 0, 200, 160 TO hDlg
    CONTROL ADD BUTTON, hDlg, 101, "Btn", 10, 10, 60, 14
    CONTROL ADD CHECKBOX, hDlg, 102, "Chk", 10, 40, 60, 14
    CONTROL HANDLE hDlg, 101 TO hBtn
    CONTROL ENABLE hDlg, 101
    CONTROL DISABLE hDlg, 101
    CONTROL HIDE hDlg, 101
    CONTROL SHOW STATE hDlg, 101, %SW_SHOW
    CONTROL SET TEXT hDlg, 101, "Renamed"
    CONTROL SET CHECK hDlg, 102, 1
    CONTROL GET CHECK hDlg, 102 TO chk
    IF hBtn = 0 THEN fail = fail + 1
    IF chk <> 1 THEN fail = fail + 1
    CONTROL KILL hDlg, 101
    IF fail = 0 THEN
        ConPrint "batch251 test ok"
    ELSE
        ConPrint "batch251 FAILURES: " & STR$(fail)
    END IF
    DIALOG END hDlg
    FUNCTION = 0
END FUNCTION

