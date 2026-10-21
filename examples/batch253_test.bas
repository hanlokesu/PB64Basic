' batch253_test.bas - CONTROL ADD SCROLLBAR/TAB txt$ slot (batch 253: official 11-arg)
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
    LOCAL hTab AS LONG
    LOCAL hSc AS LONG
    LOCAL fail AS LONG
    DIALOG NEW 0, "batch253 test", 0, 0, 200, 160 TO hDlg
    CONTROL ADD TAB, hDlg, 101, "Tab1", 10, 10, 120, 60
    CONTROL ADD SCROLLBAR, hDlg, 102, "hello", 10, 80, 100, 20
    CONTROL HANDLE hDlg, 101 TO hTab
    CONTROL HANDLE hDlg, 102 TO hSc
    IF hTab = 0 THEN fail = fail + 1
    IF hSc = 0 THEN fail = fail + 1
    IF fail = 0 THEN
        ConPrint "batch253 test ok"
    ELSE
        ConPrint "batch253 FAILURES: " & STR$(fail)
    END IF
    DIALOG END hDlg
    FUNCTION = 0
END FUNCTION

