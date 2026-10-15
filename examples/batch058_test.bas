#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

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

' PB64Basic Enhanced - batch58_test.bas
' GRAPHIC GET CANVAS / GET DC / GET MIX / SET MIX (batch 58)
FUNCTION PBMAIN() AS LONG
    LOCAL hbmp AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    LOCAL hc AS QUAD
    LOCAL hdc AS QUAD
    LOCAL mix AS LONG

    GRAPHIC BITMAP NEW 100, 50 TO hbmp
    GRAPHIC ATTACH hbmp, 0
    GRAPHIC GET CANVAS TO hc
    ConPrint "OK: canvas=" & STR$(hc)
    IF hc = hbmp THEN
        ConPrint "OK: canvas matches bitmap"
    ELSE
        ConPrint "FAIL: canvas mismatch"
        INCR fails
    END IF

    GRAPHIC GET DC TO hdc
    ConPrint "OK: dc=" & STR$(hdc)
    IF hdc <> 0 THEN
        ConPrint "OK: dc non-zero"
    ELSE
        ConPrint "FAIL: dc zero"
        INCR fails
    END IF

    GRAPHIC SET MIX (7)
    GRAPHIC GET MIX TO mix
    ConPrint "OK: mix=" & STR$(mix)
    IF mix = 7 THEN
        ConPrint "OK: mix set/get round-trip"
    ELSE
        ConPrint "FAIL: mix wrong"
        INCR fails
    END IF

    GRAPHIC SET MIX (13)
    GRAPHIC GET MIX TO mix
    IF mix = 13 THEN
        ConPrint "OK: mix reset"
    ELSE
        ConPrint "FAIL: mix reset wrong"
        INCR fails
    END IF

    GRAPHIC DETACH
    GRAPHIC BITMAP END

    IF fails = 0 THEN
        ConPrint "batch58: ALL PASS"
    ELSE
        ConPrint "batch58: FAILURES=" & STR$(fails)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
