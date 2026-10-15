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

' PB64Basic Enhanced - batch 60 test: GRAPHIC ARC / PIE / POLYLINE / PAINT
FUNCTION PBMAIN() AS LONG
    LOCAL hbmp AS LONG
    LOCAL px AS LONG
    LOCAL ok AS LONG
    LOCAL waitk AS STRING
    ok = 0
    GRAPHIC BITMAP NEW 100, 50 TO hbmp
    GRAPHIC ATTACH hbmp, 0

    ' 1. POLYLINE horizontal line
    GRAPHIC POLYLINE (10,10)-(90,10), &hFF0000
    GRAPHIC GET PIXEL (50, 10) TO px
    IF px = &hFF0000 THEN ok = ok + 1
    ConPrint "polyline px=" & STR$(px)

    ' 2. ARC full ellipse outline
    GRAPHIC ARC (0,0)-(99,49), 0, 360, &h00FF00
    GRAPHIC GET PIXEL (50, 0) TO px
    IF px = &h00FF00 THEN ok = ok + 1
    ConPrint "arc top px=" & STR$(px)

    ' 3. PIE full ellipse fill
    GRAPHIC PIE (0,0)-(99,49), 0, 360, &h0000FF, &h0000FF, 1
    GRAPHIC GET PIXEL (50, 25) TO px
    IF px = &h0000FF THEN ok = ok + 1
    ConPrint "pie center px=" & STR$(px)

    ' 4. PAINT flood fill inside a box
    GRAPHIC BOX (20,20)-(80,40), &hFFFFFF
    GRAPHIC PAINT (50, 30), &h00FF00, &hFFFFFF
    GRAPHIC GET PIXEL (50, 30) TO px
    IF px = &h00FF00 THEN ok = ok + 1
    ConPrint "paint center px=" & STR$(px)

    GRAPHIC DETACH
    IF ok = 4 THEN
        ConPrint "batch60: ALL PASS"
    ELSE
        ConPrint "batch60: FAILURES=" & STR$(4 - ok)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
