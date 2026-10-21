' batch245_test.bas - CONTROL ADD LISTBOX official syntax (batch 245: items$() array)
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
    LOCAL hLB AS LONG
    LOCAL fail AS LONG
    DIM items() AS STRING
    DIM items(1 TO 3)
    items(1) = "Alpha"
    items(2) = "Beta"
    items(3) = "Gamma"
    DIALOG NEW 0, "batch245 test", 0, 0, 200, 160 TO hDlg
    CONTROL ADD LISTBOX, hDlg, 105, items(), 10, 10, 90, 70
    CONTROL HANDLE hDlg, 105 TO hLB
    IF hLB = 0 THEN fail = fail + 1
    IF fail = 0 THEN
        ConPrint "batch245 test ok"
    ELSE
        ConPrint "batch245 FAILURES: " & STR$(fail)
    END IF
    DIALOG END hDlg
    FUNCTION = 0
END FUNCTION

