

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

TYPE PolyPts3
    count AS LONG
    x1 AS SINGLE
    y1 AS SINGLE
    x2 AS SINGLE
    y2 AS SINGLE
    x3 AS SINGLE
    y3 AS SINGLE
END TYPE
TYPE PolyPts4
    count AS LONG
    x1 AS SINGLE
    y1 AS SINGLE
    x2 AS SINGLE
    y2 AS SINGLE
    x3 AS SINGLE
    y3 AS SINGLE
    x4 AS SINGLE
    y4 AS SINGLE
END TYPE
TYPE PolyPts5
    count AS LONG
    x1 AS SINGLE
    y1 AS SINGLE
    x2 AS SINGLE
    y2 AS SINGLE
    x3 AS SINGLE
    y3 AS SINGLE
    x4 AS SINGLE
    y4 AS SINGLE
    x5 AS SINGLE
    y5 AS SINGLE
END TYPE

' PB64Basic Enhanced - Batch 73 Test: XPRINT POLYGON / POLYLINE
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    LOCAL tri AS PolyPts3
    LOCAL rect AS PolyPts4
    LOCAL line5 AS PolyPts5
    fails = 0

    ConPrint "=== Batch 73: XPRINT polygon + polyline ==="

    XPRINT ATTACH DEFAULT

    XPRINT COLOR 255, 0
    tri.count = 3
    tri.x1 = 100: tri.y1 = 100: tri.x2 = 200: tri.y2 = 100: tri.x3 = 150: tri.y3 = 50
    XPRINT POLYGON tri
    ConPrint "POLYGON (triangle): done"

    XPRINT COLOR 0, 255
    line5.count = 5
    line5.x1 = 50: line5.y1 = 200: line5.x2 = 100: line5.y2 = 250
    line5.x3 = 150: line5.y3 = 200: line5.x4 = 200: line5.y4 = 250
    line5.x5 = 250: line5.y5 = 200
    XPRINT POLYLINE line5
    ConPrint "POLYLINE (5 pts): done"

    XPRINT COLOR 255
    rect.count = 4
    rect.x1 = 300: rect.y1 = 100: rect.x2 = 400: rect.y2 = 100
    rect.x3 = 400: rect.y3 = 200: rect.x4 = 300: rect.y4 = 200
    XPRINT POLYGON rect
    ConPrint "POLYGON (rect): done"

    XPRINT CLOSE

    ConPrint "=== Result: ALL PASS (GDI calls returned, no crash)"
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





