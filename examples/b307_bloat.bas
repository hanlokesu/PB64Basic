#COMPILE EXE
#DIM ALL
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' _b307_bloat.bas - legacy #16: #BLOAT acceptance.
' PBWin10 /L /Q rejects any #BLOAT value with Error 496 (Destination file
' write error, incl. the 4 MB CHW sample) - an environment hard limit.  The
' fork accepts and ignores the directive, so the real probe lives in the
' #ELSE (fork) branch; PBWin compiles only the stub.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PBWin stub: #BLOAT rejected (496) on /L /Q - fork-only probe.
END FUNCTION
#ELSE
#BLOAT 4096
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB
FUNCTION PBMAIN() AS LONG
    ConPrint "bloat=ok" & $CRLF
    FUNCTION = 0
END FUNCTION
#ENDIF
