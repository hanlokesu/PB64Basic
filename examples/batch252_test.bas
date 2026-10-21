' batch252_test.bas - COMBOBOX verbs (batch 252: ADD/SELECT/GET TEXT/COUNT/RESET)
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
    LOCAL hCB AS LONG
    LOCAL tx AS STRING
    LOCAL cnt AS LONG
    LOCAL fail AS LONG
    DIALOG NEW 0, "batch252 test", 0, 0, 200, 160 TO hDlg
    LOCAL items() AS STRING
    DIM items(1)
    items(0) = "One"
    items(1) = "Two"
    CONTROL ADD COMBOBOX, hDlg, 101, items(), 10, 10, 100, 120
    CONTROL HANDLE hDlg, 101 TO hCB
    COMBOBOX ADD hDlg, 101, "Three"
    COMBOBOX SELECT hDlg, 101, 2
    COMBOBOX GET TEXT hDlg, 101 TO tx
    COMBOBOX GET COUNT hDlg, 101 TO cnt
    IF hCB = 0 THEN fail = fail + 1
    IF tx <> "Three" THEN fail = fail + 1
    IF cnt <> 3 THEN fail = fail + 1
    COMBOBOX RESET hDlg, 101
    IF fail = 0 THEN
        ConPrint "batch252 test ok"
    ELSE
        ConPrint "batch252 FAILURES: " & STR$(fail)
    END IF
    DIALOG END hDlg
    FUNCTION = 0
END FUNCTION

