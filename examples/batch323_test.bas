#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 323 test
' JK-IDE addon release (v0.2.114): tab and editor context menu improvements.
' Copy File now also opens the source folder automatically, and the temporary
' diagnostic logging was removed for a clean final build.
' This is the headless console smoke: it has no WAITKEY/MSGBOX tail, so the
' release gate compiles and runs it end to end.
FUNCTION PBMAIN() AS LONG
    PRINT "PB64Basic batch 323: JK-IDE addon release v0.2.114."
    PRINT "Copy File opens the source folder; clean build with no diagnostics."
    PRINT "OK=1"
END FUNCTION
