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

' PB64Basic Enhanced - Batch 63 test
' GRAPHIC GET CLIP / GET VIEW / SET VIEW / GET LINES / GET+SET WRAP
' All statements verified against a memory DIB bitmap (console-testable).
FUNCTION PBMAIN() AS LONG
    LOCAL hBmp AS LONG
    LOCAL cw AS SINGLE
    LOCAL ch AS SINGLE
    LOCAL vx AS SINGLE
    LOCAL vy AS SINGLE
    LOCAL lines AS LONG
    LOCAL wrapv AS LONG
    LOCAL fails AS LONG
    LOCAL waitk AS STRING

    fails = 0

    ' Create a 100 x 50 memory bitmap and attach it
    GRAPHIC BITMAP NEW 100, 50 TO hBmp
    GRAPHIC ATTACH hBmp, 0
    ' 1. GRAPHIC GET CLIP TO w!, h!  - default clip area is the whole bitmap
    GRAPHIC GET CLIP TO cw, ch
    IF cw <> 100 OR ch <> 50 THEN
        ConPrint "FAIL 1: GET CLIP = " & STR$(cw) & "x" & STR$(ch) & " (expected 100x50)"
        fails = fails + 1
    ELSE
        ConPrint "OK 1: GET CLIP = 100x50"
    END IF

    ' 2. GRAPHIC GET VIEW default -> 0, 0
    GRAPHIC GET VIEW TO vx, vy
    IF vx <> 0 OR vy <> 0 THEN
        ConPrint "FAIL 2: GET VIEW default = " & STR$(vx) & "," & STR$(vy) & " (expected 0,0)"
        fails = fails + 1
    ELSE
        ConPrint "OK 2: GET VIEW default = 0,0"
    END IF

    ' 3. GRAPHIC SET VIEW 20, 30 then GET VIEW -> 20, 30
    GRAPHIC SET VIEW 20, 30
    GRAPHIC GET VIEW TO vx, vy
    IF vx <> 20 OR vy <> 30 THEN
        ConPrint "FAIL 3: GET VIEW after SET VIEW = " & STR$(vx) & "," & STR$(vy) & " (expected 20,30)"
        fails = fails + 1
    ELSE
        ConPrint "OK 3: SET VIEW 20,30 -> GET VIEW = 20,30"
    END IF

    ' 4. GRAPHIC GET LINES TO n& - bitmap height = 50 lines
    GRAPHIC GET LINES TO lines
    IF lines <> 50 THEN
        ConPrint "FAIL 4: GET LINES = " & STR$(lines) & " (expected 50)"
        fails = fails + 1
    ELSE
        ConPrint "OK 4: GET LINES = 50"
    END IF

    ' 5. GRAPHIC SET WRAP 0 then GET WRAP -> 0
    GRAPHIC SET WRAP 0
    GRAPHIC GET WRAP TO wrapv
    IF wrapv <> 0 THEN
        ConPrint "FAIL 5: GET WRAP after SET WRAP 0 = " & STR$(wrapv)
        fails = fails + 1
    ELSE
        ConPrint "OK 5: SET WRAP 0 -> GET WRAP = 0"
    END IF

    ' 6. GRAPHIC SET WRAP 1 then GET WRAP -> 1
    GRAPHIC SET WRAP 1
    GRAPHIC GET WRAP TO wrapv
    IF wrapv <> 1 THEN
        ConPrint "FAIL 6: GET WRAP after SET WRAP 1 = " & STR$(wrapv)
        fails = fails + 1
    ELSE
        ConPrint "OK 6: SET WRAP 1 -> GET WRAP = 1"
    END IF

    GRAPHIC DETACH
    GRAPHIC BITMAP END
    IF fails = 0 THEN
        ConPrint "ALL PASS (6/6)"
    ELSE
        ConPrint "TOTAL FAILS: " & STR$(fails)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

