#COMPILE EXE
#DIM ALL

' batch 357 self-check: DIALOG NEW must create the dialog HIDDEN (PBWin
' semantics), DIALOG SHOW MODELESS must reveal it, DIALOG SHOW STATE 3 must
' maximize it and STATE 1 must restore it.  Plain Win32 IsWindowVisible /
' IsZoomed read-back keeps the assertion independent of the runtime's own
' bookkeeping.

DECLARE FUNCTION IsWindowVisible LIB "USER32.DLL" ALIAS "IsWindowVisible" (BYVAL hWnd AS LONG) AS LONG
DECLARE FUNCTION IsZoomed LIB "USER32.DLL" ALIAS "IsZoomed" (BYVAL hWnd AS LONG) AS LONG
DECLARE FUNCTION Sleep LIB "KERNEL32.DLL" ALIAS "Sleep" (BYVAL ms AS DWORD) AS LONG

FUNCTION PBMAIN () AS LONG
    LOCAL h AS LONG
    LOCAL vis0 AS LONG, vis1 AS LONG, vis2 AS LONG, vis3 AS LONG
    LOCAL zoom1 AS LONG, zoom2 AS LONG
    LOCAL fails AS LONG

    fails = 0
    DIALOG NEW 0, "b357 dialog semantics", 100, 100, 200, 80 TO h
    vis0 = IsWindowVisible(h)
    PRINT "after DIALOG NEW      : visible="; vis0; "  (want 0)"

    DIALOG SHOW MODELESS h
    Sleep 50
    vis1 = IsWindowVisible(h)
    zoom1 = IsZoomed(h)
    PRINT "after SHOW MODELESS   : visible="; vis1; " zoomed="; zoom1; "  (want visible=1 zoomed=0)"

    DIALOG SHOW STATE h, 3
    Sleep 50
    vis2 = IsWindowVisible(h)
    zoom2 = IsZoomed(h)
    PRINT "after SHOW STATE 3    : visible="; vis2; " zoomed="; zoom2; "  (want visible=1 zoomed=1)"

    DIALOG SHOW STATE h, 1
    Sleep 50
    vis3 = IsWindowVisible(h)
    PRINT "after SHOW STATE 1    : visible="; vis3; "  (want 1)"

    DIALOG END h

    IF vis0 <> 0 THEN
        PRINT "FAIL: DIALOG NEW created a visible window"
        fails = fails + 1
    END IF
    IF vis1 = 0 THEN
        PRINT "FAIL: DIALOG SHOW MODELESS did not reveal the dialog"
        fails = fails + 1
    END IF
    IF zoom2 = 0 THEN
        PRINT "FAIL: DIALOG SHOW STATE 3 did not maximize"
        fails = fails + 1
    END IF
    IF vis3 = 0 THEN
        PRINT "FAIL: DIALOG SHOW STATE 1 hid the dialog"
        fails = fails + 1
    END IF
    PRINT "=== FAILURES: "; fails
    FUNCTION = 0
END FUNCTION
