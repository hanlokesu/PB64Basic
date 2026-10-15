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

' Tier-3 DDT: CONTROL ADD BUTTON - button click test
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)
FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hBtn AS QUAD
    MSGBOX "Batch 130: Tests CONTROL ADD BUTTON - creates a push button on the window.", 0, "Test: BUTTON"
    WINDOW "Batch 130: Button", 100, 100, 400, 300 TO hWnd
    CONTROL ADD BUTTON, hWnd, 100, "Click Me!", 20, 20, 100, 30 TO hBtn
    MSGBOX "Window + Button created. hBtn=" + STR$(hBtn), 0, "BUTTON OK"
    pb_message_loop
END FUNCTION


#ENDIF
