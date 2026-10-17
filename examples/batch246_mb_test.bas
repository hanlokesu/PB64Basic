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

FUNCTION PBMAIN() AS LONG
    LOCAL fail AS LONG
    IF %MB_ABORTRETRYIGNORE <> 2 THEN fail = fail + 1
    IF %MB_RETRYCANCEL <> 5 THEN fail = fail + 1
    IF %MB_CANCELTRYCONTINUE <> 6 THEN fail = fail + 1
    IF %MB_DEFBUTTON4 <> 768 THEN fail = fail + 1
    IF %MB_ICONHAND <> 16 THEN fail = fail + 1
    IF %MB_ICONSTOP <> 16 THEN fail = fail + 1
    IF %MB_ICONEXCLAMATION <> 48 THEN fail = fail + 1
    IF %MB_ICONASTERISK <> 64 THEN fail = fail + 1
    IF %MB_APPLMODAL <> 0 THEN fail = fail + 1
    IF %MB_SYSTEMMODAL <> 4096 THEN fail = fail + 1
    IF %MB_TASKMODAL <> 8192 THEN fail = fail + 1
    IF %MB_HELP <> 16384 THEN fail = fail + 1
    IF %MB_NOFOCUS <> 32768 THEN fail = fail + 1
    IF %MB_SETFOREGROUND <> 65536 THEN fail = fail + 1
    IF %MB_DEFAULT_DESKTOP_ONLY <> 131072 THEN fail = fail + 1
    IF %MB_TOPMOST <> 262144 THEN fail = fail + 1
    IF %MB_RIGHT <> 524288 THEN fail = fail + 1
    IF %MB_RTLREADING <> 1048576 THEN fail = fail + 1
    IF %MB_SERVICE_NOTIFICATION <> 2097152 THEN fail = fail + 1
    IF %IDTRYAGAIN <> 10 THEN fail = fail + 1
    IF %IDCONTINUE <> 11 THEN fail = fail + 1
    ConPrint "=== FAILURES: " & STR$(fail) & " ==="
    FUNCTION = fail
END FUNCTION