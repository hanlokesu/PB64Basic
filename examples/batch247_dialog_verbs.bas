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

' batch247: DIALOG HIDE/MAXIMIZE/MINIMIZE/NORMALIZE/DISABLE + DIALOG FONT legacy
' CHW: DIALOG_HIDE_statement.htm / DIALOG_MAXIMIZE_statement.htm / DIALOG_MINIMIZE_statement.htm
'      DIALOG_NORMALIZE_statement.htm / dialog_disable.htm / dialog_font.htm
FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG
    DIALOG FONT "MS Sans Serif", 8
    DIALOG NEW 0, "batch247", 10, 10, 200, 100 TO hDlg
    DIALOG HIDE hDlg
    DIALOG MAXIMIZE hDlg
    DIALOG MINIMIZE hDlg
    DIALOG NORMALIZE hDlg
    DIALOG DISABLE hDlg
    DIALOG ENABLE hDlg
    DIALOG DEFAULT FONT "Arial", 10, 0, 1
    DIALOG END hDlg
    ConPrint "dialog verbs ok"
END FUNCTION