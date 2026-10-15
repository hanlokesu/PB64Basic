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

' PB64Basic Enhanced - batch56_test.bas
' GRAPHIC CIRCLE / POLYGON / GET CLIENT / GET LOC (batch 56)
FUNCTION PBMAIN() AS LONG
    LOCAL hbmp AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    LOCAL px AS LONG
    LOCAL cw AS LONG
    LOCAL ch AS LONG
    LOCAL lx AS LONG
    LOCAL ly AS LONG

    GRAPHIC BITMAP NEW 100, 50 TO hbmp
    IF hbmp = 0 THEN
        ConPrint "FAIL: bitmap handle is zero"
        INCR fails
    ELSE
        ConPrint "OK: bitmap handle " & STR$(hbmp)
    END IF

    GRAPHIC ATTACH hbmp, 0
    GRAPHIC CIRCLE (50, 25), 15, 255
    ConPrint "OK: circle drawn"

    GRAPHIC GET PIXEL (50, 25) TO px
    ConPrint "OK: center pixel=" & STR$(px)
    IF px <> 0 THEN
        ConPrint "OK: circle center non-zero"
    ELSE
        ConPrint "FAIL: circle center is zero"
        INCR fails
    END IF

    GRAPHIC POLYGON (10,10)-(30,10)-(20,30), 255
    ConPrint "OK: polygon drawn"

    GRAPHIC GET PIXEL (20, 20) TO px
    ConPrint "OK: polygon pixel=" & STR$(px)
    IF px <> 0 THEN
        ConPrint "OK: polygon interior non-zero"
    ELSE
        ConPrint "FAIL: polygon interior is zero"
        INCR fails
    END IF

    GRAPHIC GET CLIENT TO cw, ch
    ConPrint "OK: client=" & STR$(cw) & "x" & STR$(ch)
    IF cw = 100 AND ch = 50 THEN
        ConPrint "OK: client size correct"
    ELSE
        ConPrint "FAIL: client size wrong"
        INCR fails
    END IF

    GRAPHIC GET LOC TO lx, ly
    ConPrint "OK: loc=" & STR$(lx) & "," & STR$(ly)
    IF lx = 0 AND ly = 0 THEN
        ConPrint "OK: loc correct"
    ELSE
        ConPrint "FAIL: loc wrong"
        INCR fails
    END IF

    GRAPHIC DETACH
    GRAPHIC BITMAP END

    IF fails = 0 THEN
        ConPrint "batch56: ALL PASS"
    ELSE
        ConPrint "batch56: FAILURES=" & STR$(fails)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
