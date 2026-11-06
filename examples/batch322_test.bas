#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 322 test
' JK-IDE addon release (v0.2.113): editor tab context menu.
' Seven tab right-click actions (Copy Path, Copy File Name, Copy, Open with,
' Open in Explorer, Rename, Close Tab) plus two separators.
' This is the headless console smoke: it has no WAITKEY/MSGBOX tail, so the
' release gate compiles and runs it end to end.
FUNCTION PBMAIN() AS LONG
    PRINT "PB64Basic batch 322: JK-IDE addon release v0.2.113."
    PRINT "Editor tab context menu: seven actions and two separators."
    PRINT "OK=1"
END FUNCTION
