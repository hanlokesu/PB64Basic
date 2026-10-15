#COMPILE EXE
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

' Tier-3 DDT: BUTTON - button click action
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


SUB OnBtn1
    MSGBOX "Button 1 clicked!", 0, "Batch 139"
END SUB

SUB OnBtn2
    MSGBOX "Button 2 clicked!", 0, "Batch 139"
END SUB

SUB OnBtn3
    MSGBOX "Button 3 clicked!", 0, "Batch 139"
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hBtn1 AS QUAD
    LOCAL hBtn2 AS QUAD
    LOCAL hBtn3 AS QUAD

    WINDOW "Batch 139: CONTROL CMD", 100, 100, 350, 250 TO hWnd

    CONTROL ADD BUTTON, hWnd, 101, "Click Me 1",  30, 30, 120, 35 TO hBtn1
    CONTROL ADD BUTTON, hWnd, 102, "Click Me 2",  30, 80, 120, 35 TO hBtn2
    CONTROL ADD BUTTON, hWnd, 103, "Click Me 3",  30, 130, 120, 35 TO hBtn3

    CONTROL CMD hBtn1, OnBtn1
    CONTROL CMD hBtn2, OnBtn2
    CONTROL CMD hBtn3, OnBtn3

    MESSAGE LOOP
END FUNCTION


#ENDIF
