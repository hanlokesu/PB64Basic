#COMPILE EXE
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF
#COMPILER PBWIN 10

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

' batch248: CONTROL ADD COMBOBOX full official syntax
' CHW control_add_combobox.htm:
'   CONTROL ADD COMBOBOX, hDlg, id&, [items$()], x, y, xx, yy
'     [, [style&] [, [exstyle&]]] [[,] CALL callback]
FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG
    LOCAL hCbo AS LONG
    LOCAL hLb AS LONG
    LOCAL n AS LONG
    LOCAL m AS LONG
    LOCAL s AS STRING
    LOCAL res AS STRING
    LOCAL fail AS LONG
    LOCAL items() AS STRING
    DIM items(1 TO 3)
    items(1) = "Alpha"
    items(2) = "Beta"
    items(3) = "Gamma"
    DIALOG NEW 0, "batch248", 10, 10, 260, 120 TO hDlg
    CONTROL ADD COMBOBOX, hDlg, 201, items(), 20, 10, 160, 80
    CONTROL HANDLE hDlg, 201 TO hCbo
    IF hCbo = 0 THEN fail = fail + 1
    COMBOBOX GET COUNT hDlg, 201 TO n
    IF n <> 3 THEN fail = fail + 1
    COMBOBOX GET TEXT hDlg, 201, 1 TO s
    IF s <> "Alpha" THEN fail = fail + 1
    CONTROL ADD COMBOBOX, hDlg, 202, , 20, 40, 160, 80, &H10000
    CONTROL ADD LISTBOX, hDlg, 203, items(), 190, 10, 60, 60
    LISTBOX GET COUNT hDlg, 203 TO m
    IF m <> 3 THEN fail = fail + 1
    DIALOG END hDlg
    IF fail = 0 THEN
        res = "combobox items ok"
    ELSE
        res = "FAILURES: " + STR$(fail) + " hCbo=" + STR$(hCbo) + " n=" + STR$(n) + " m=" + STR$(m) + " s=[" + s + "]"
    END IF
    ConPrint res
    OPEN "batch248_result.txt" FOR OUTPUT AS #1
    PRINT #1, res
    CLOSE #1
END FUNCTION