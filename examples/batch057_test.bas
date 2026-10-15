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

' PB64Basic Enhanced - batch57_test.bas
' GRAPHIC BITMAP LOAD / CHR SIZE / CELL / CELL SIZE (batch 57)
FUNCTION PBMAIN() AS LONG
    LOCAL hbmp AS LONG
    LOCAL hbmp2 AS QUAD
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    LOCAL px AS LONG
    LOCAL cw AS LONG
    LOCAL ch AS LONG
    LOCAL csw AS LONG
    LOCAL csh AS LONG
    LOCAL cx AS LONG
    LOCAL cy AS LONG

    GRAPHIC BITMAP NEW 100, 50 TO hbmp
    GRAPHIC ATTACH hbmp, 0
    GRAPHIC BOX (10,10)-(40,40), 255, 255, 1
    GRAPHIC SAVE "batch57_out.bmp"
    GRAPHIC DETACH
    GRAPHIC BITMAP END

    GRAPHIC BITMAP LOAD "batch57_out.bmp" TO hbmp2
    IF hbmp2 <> 0 THEN
        ConPrint "OK: bitmap loaded"
    ELSE
        ConPrint "FAIL: bitmap load failed"
        INCR fails
    END IF

    GRAPHIC ATTACH hbmp2
    GRAPHIC GET PIXEL (25, 25) TO px
    ConPrint "OK: loaded pixel=" & STR$(px)
    IF px <> 0 THEN
        ConPrint "OK: loaded bitmap has content"
    ELSE
        ConPrint "FAIL: loaded bitmap empty"
        INCR fails
    END IF

    GRAPHIC CHR SIZE ("ABC") TO cw, ch
    ConPrint "OK: chr size=" & STR$(cw) & "x" & STR$(ch)
    IF cw > 0 AND ch > 0 THEN
        ConPrint "OK: chr size positive"
    ELSE
        ConPrint "FAIL: chr size wrong"
        INCR fails
    END IF

    GRAPHIC CELL SIZE (2, 3) TO csw, csh
    ConPrint "OK: cell size=" & STR$(csw) & "x" & STR$(csh)
    IF csw > 0 AND csh > 0 THEN
        ConPrint "OK: cell size positive"
    ELSE
        ConPrint "FAIL: cell size wrong"
        INCR fails
    END IF

    GRAPHIC CELL (2, 3) TO cx, cy
    ConPrint "OK: cell=" & STR$(cx) & "," & STR$(cy)
    IF cx > 0 AND cy > 0 THEN
        ConPrint "OK: cell positive"
    ELSE
        ConPrint "FAIL: cell wrong"
        INCR fails
    END IF

    GRAPHIC DETACH
    GRAPHIC BITMAP END
    KILL "batch57_out.bmp"

    IF fails = 0 THEN
        ConPrint "batch57: ALL PASS"
    ELSE
        ConPrint "batch57: FAILURES=" & STR$(fails)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
