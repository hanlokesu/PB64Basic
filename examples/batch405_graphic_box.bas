#COMPILE EXE
#DIM ALL

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
    IF h <> 0 THEN WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

SUB CheckPixel(BYVAL x AS LONG, BYVAL y AS LONG, BYVAL want AS LONG, BYREF failures AS LONG)
    LOCAL px AS LONG
    GRAPHIC GET PIXEL (x, y) TO px
    IF px <> want THEN
        failures = failures + 1
        ConPrint "pixel(" & STR$(x) & "," & STR$(y) & ") got=" & STR$(px) & " want=" & STR$(want)
    END IF
END SUB

' PB64Basic Enhanced - batch 405: GRAPHIC BOX negative corner semantics.
' PBWin maps any negative corner to 100% roundness; verified with a
' PBWin pixel probe covering -1/-10/-50/-200.
FUNCTION PBMAIN() AS LONG
    LOCAL hbmp AS LONG
    LOCAL failures AS LONG
    failures = 0
    GRAPHIC BITMAP NEW 900, 300 TO hbmp
    GRAPHIC ATTACH hbmp, 0
    GRAPHIC CLEAR RGB(255, 255, 255)

    GRAPHIC BOX (10, 10)-(90, 90), 0, RGB(0, 0, 0), RGB(255, 0, 0)
    GRAPHIC BOX (110, 10)-(190, 90), 10, RGB(0, 0, 0), RGB(0, 255, 0)
    GRAPHIC BOX (210, 10)-(290, 90), -10, RGB(0, 0, 0), RGB(0, 0, 255)
    GRAPHIC BOX (310, 10)-(390, 90), -1, RGB(0, 0, 0), RGB(255, 255, 0)
    GRAPHIC BOX (410, 10)-(490, 90), -200, RGB(0, 0, 0), RGB(255, 0, 255)
    GRAPHIC BOX (510, 10)-(590, 90), 100, RGB(0, 0, 0), RGB(0, 255, 255)

    ' Non-square boxes and corner values above 100.  PBWin treats corner
    ' 200 as 100% and uses min(width,height) as the round diameter; these
    ' boxes match the PBWin aspect probe pixel for pixel.
    GRAPHIC BOX (10, 110)-(170, 190), 100, RGB(0, 0, 0), RGB(0, 128, 255)
    GRAPHIC BOX (230, 110)-(390, 190), 200, RGB(0, 0, 0), RGB(255, 128, 0)
    GRAPHIC BOX (450, 110)-(530, 270), 100, RGB(0, 0, 0), RGB(128, 0, 128)
    GRAPHIC BOX (570, 110)-(650, 270), 200, RGB(0, 0, 0), RGB(0, 128, 0)
    GRAPHIC BOX (690, 110)-(850, 190), -10, RGB(0, 0, 0), RGB(128, 128, 0)
    GRAPHIC BOX (690, 210)-(850, 290), -200, RGB(0, 0, 0), RGB(0, 128, 128)

    ' corner 0 is a sharp corner: the interior reaches the top-left pixel.
    CheckPixel 12, 12, RGB(255, 0, 0), failures

    ' +10 keeps a small radius, still filled two pixels inside the corner.
    CheckPixel 112, 12, RGB(0, 255, 0), failures

    ' negative corners are fully rounded: top-left 2x2 is background.
    CheckPixel 212, 12, RGB(255, 255, 255), failures
    CheckPixel 312, 12, RGB(255, 255, 255), failures
    CheckPixel 412, 12, RGB(255, 255, 255), failures

    ' +100 is the explicit full-round value and must match negative corners.
    CheckPixel 512, 12, RGB(255, 255, 255), failures

    ' Non-square 160x80 and 80x160 boxes: the round diameter is the shorter
    ' side, so the top-left 2x2 is background while the mid-left edge is filled.
    CheckPixel 12, 112, RGB(255, 255, 255), failures
    CheckPixel 12, 150, RGB(0, 128, 255), failures
    CheckPixel 232, 112, RGB(255, 255, 255), failures
    CheckPixel 232, 150, RGB(255, 128, 0), failures
    CheckPixel 452, 112, RGB(255, 255, 255), failures
    CheckPixel 452, 190, RGB(128, 0, 128), failures
    CheckPixel 572, 112, RGB(255, 255, 255), failures
    CheckPixel 572, 190, RGB(0, 128, 0), failures

    ' Negative corners on non-square boxes also mean 100%.
    CheckPixel 692, 112, RGB(255, 255, 255), failures
    CheckPixel 692, 150, RGB(128, 128, 0), failures
    CheckPixel 692, 212, RGB(255, 255, 255), failures
    CheckPixel 692, 250, RGB(0, 128, 128), failures

    ' Reverse coordinates still normalise width/height and keep the corner.
    GRAPHIC CLEAR RGB(255, 255, 255)
    GRAPHIC BOX (90, 10)-(10, 90), -10, RGB(0, 0, 0), RGB(128, 128, 128)
    CheckPixel 12, 12, RGB(255, 255, 255), failures

    ' Omitted corner (two commas) must stay a sharp rectangle; only an
    ' explicit negative corner means 100%.  PBWin baseline:
    ' omitted 12,12 = fill, explicit 0 112,12 = fill, -1 212,12 = white.
    GRAPHIC CLEAR RGB(255, 255, 255)
    GRAPHIC BOX (10, 10)-(90, 90), , RGB(0, 0, 0), RGB(0, 128, 0)
    GRAPHIC BOX (110, 10)-(190, 90), 0, RGB(0, 0, 0), RGB(0, 128, 0)
    GRAPHIC BOX (210, 10)-(290, 90), -1, RGB(0, 0, 0), RGB(0, 128, 0)
    CheckPixel 12, 12, RGB(0, 128, 0), failures
    CheckPixel 11, 11, RGB(0, 128, 0), failures
    CheckPixel 112, 12, RGB(0, 128, 0), failures
    CheckPixel 212, 12, RGB(255, 255, 255), failures

    GRAPHIC DETACH
    IF failures = 0 THEN
        ConPrint "batch405: ALL PASS"
    ELSE
        ConPrint "batch405: FAILURES=" & STR$(failures)
    END IF
    FUNCTION = failures
END FUNCTION
