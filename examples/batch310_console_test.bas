#COMPILE EXE
#DIM ALL
#CONSOLE ON

' PB64Basic Enhanced - batch 310 console-subsystem test
' #CONSOLE ON must link /SUBSYSTEM:CONSOLE so PRINT reaches the console.
' Headless: no WAITKEY/MSGBOX tail.
FUNCTION PBMAIN() AS LONG
    PRINT "CONSOLE_OK=1"
    FUNCTION = 0
END FUNCTION
