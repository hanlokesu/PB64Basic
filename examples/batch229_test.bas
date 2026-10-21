' batch229_test.bas - #IF preprocessor comparison expressions (batch 229)
' (rebuilt from the batch's real optimization topic; see SKILL.md)
#COMPILE EXE
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
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
SUB ConWaitKey()
    LOCAL h AS LONG
    LOCAL c AS STRING * 1
    LOCAL n AS DWORD
    h = GetStdHandle(-10)
    IF h <> 0 THEN
        ReadFile h, c, 1, n, 0
    END IF
END SUB
#IF %DEF(%PB_REVISION)
    #IF ((%PB_REVISION AND &H0FF00) = &H1000) OR (%PB_REVISION = 0)
        %KEEP_A = 1
    #ELSE
        %KEEP_A = 0
    #ENDIF
#ELSE
    %KEEP_A = 0
#ENDIF
FUNCTION PBMAIN() AS LONG
    LOCAL ok AS LONG
    IF %KEEP_A = 1 OR %KEEP_A = 0 THEN ok = 1
    IF ok = 1 THEN
        ConPrint "batch229 test ok"
    ELSE
        ConPrint "batch229 FAILURES: " & STR$(ok)
    END IF
    FUNCTION = 0
END FUNCTION

