#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 309 test
' JK-IDE addon release (v0.2.100): inline error annotation, save-before-compile,
' error-line jump, unified red output and the RUN-path fatal-error fix.
' This is the headless console smoke: it has no WAITKEY/MSGBOX tail, so the
' release gate compiles and runs it end to end.
FUNCTION PBMAIN() AS LONG
    PRINT "PB64Basic batch 309: JK-IDE addon release v0.2.100."
    PRINT "Inline error annotation, save-before-compile, error-line jump."
    PRINT "OK=1"
END FUNCTION