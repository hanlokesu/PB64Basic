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

' Tier-3 DDT: CONTROL ADD RADIOBUTTON - radio selection
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)
FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hR AS QUAD
    MSGBOX "Batch 136: Tests CONTROL ADD RADIOBUTTON - 3 radio buttons (select one).", 0, "Test: RADIOBUTTON"
    WINDOW "Batch 136: RadioButton", 200, 200, 500, 400 TO hWnd
    CONTROL ADD RADIOBUTTON, hWnd, 101, "Option A", 20, 20, 400, 30 TO hR
    CONTROL ADD RADIOBUTTON, hWnd, 102, "Option B", 20, 60, 400, 30 TO hR
    CONTROL ADD RADIOBUTTON, hWnd, 103, "Option C", 20, 100, 400, 30 TO hR
    pb_message_loop
END FUNCTION


#ENDIF
