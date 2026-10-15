

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.



' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
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

' batch 199c witness - handles held in QUAD (pointer-sized on x64)
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

FUNCTION PBMAIN () AS LONG
    #IF %MY_PBVER = 10
        LOCAL hil AS LONG
    #ELSE
        LOCAL hil AS QUAD
    #ENDIF

    #IF %MY_PBVER = 10
        LOCAL hic AS LONG
    #ELSE
        LOCAL hic AS QUAD
    #ENDIF

    LOCAL cnt AS LONG
    LOCAL k AS LONG
    ConPrint "=== IMAGELIST witness (batch 199c) ==="
    IMAGELIST NEW BITMAP 16, 16, 32, 4 TO hil
    IMAGELIST NEW ICON 16, 16, 32, 4 TO hic
    ConPrint "handle bitmap list = nonzero (value differs by bitness)"
    ConPrint "handle icon   list = nonzero (value differs by bitness)"
    IMAGELIST GET COUNT hil TO cnt
    ConPrint "GET COUNT hil      = " & STR$(cnt)
    IMAGELIST KILL hil
    IMAGELIST KILL hic
    ConPrint "killed both; done"
    FUNCTION = 0
' Press any key to exit...
ConWaitKey
END FUNCTION





