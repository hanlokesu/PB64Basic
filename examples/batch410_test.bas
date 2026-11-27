#COMPILE EXE
#DIM ALL
#INCLUDE "win32api.inc"
#INCLUDE "commctrl.inc"

' batch 410 regression: three-layer Gary Beene findings that are cheap to
' assert at compile time.  The custom-draw return path and owner-data text
' are exercised by the batch-3 harness screenshots; this example pins the
' parser and constant semantics.

SUB GraphicsColonProbe()
    GRAPHIC CLEAR : GRAPHIC REDRAW
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS DWORD
    LOCAL fails AS LONG

    ' Empty style slot and empty exstyle slot in DIALOG NEW.
    DIALOG NEW PIXELS, 0, "b410a", 0, 0, 200, 100, %WS_OverlappedWindow, , TO hDlg
    DIALOG END hDlg
    DIALOG NEW PIXELS, 0, "b410b", 0, 0, 200, 100, , %WS_EX_ToolWindow TO hDlg
    DIALOG END hDlg

    ' PB64 creates ANSI common controls; the unsuffixed owner-data alias is A.
    IF %RGB_PALETURQUOISE <> &H00EEEEAF THEN fails = fails + 1
    IF %RGB_GREEN <> &H00008000 THEN fails = fails + 1
    IF %RGB_LIGHTBLUE <> &H00E6D8AD THEN fails = fails + 1
    IF %LVN_GETDISPINFO <> -150 THEN fails = fails + 1
    IF %LVN_GETDISPINFOW <> -177 THEN fails = fails + 1

    PRINT "FAILURES="; fails
    FUNCTION = fails
END FUNCTION
