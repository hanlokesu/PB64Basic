' batch394_test.bas - Windows regression for the Android-only runtime branches.
' BEEP and PLAY SOUND must still compile and link on Windows after the JNI
' extern declarations were added. Android MSGBOX is not called here.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL failures AS LONG

    BEEP
    PLAY SOUND 440, 40

    PRINT "batch394 failures=" + STR$(failures)
    FUNCTION = failures
END FUNCTION