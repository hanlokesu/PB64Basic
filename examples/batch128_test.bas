#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

' Tier-3 DDT: WINDOW NEW - basic window creation
' Auto-generated: PB64Basic fork batch test

FUNCTION PBMAIN() AS LONG
    LOCAL h AS QUAD
    MSGBOX "Batch 128: Tests WINDOW statement - creates a native Win32 window.", 0, "Test: WINDOW"
    WINDOW "Window Test", 100, 100, 400, 300 TO h
    MSGBOX "Window created (hwnd=" + STR$(h) + "). Close this message to end.", 0, "WINDOW OK"
END FUNCTION


#ENDIF
