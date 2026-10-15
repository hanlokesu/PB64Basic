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

' Tier-3 DDT: CONTROL ADD CHECKBOX - toggle checkbox
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)
FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hChk AS QUAD
    MSGBOX "Batch 135: Tests CONTROL ADD CHECKBOX - 3 checkboxes. Click them to toggle.", 0, "Test: CHECKBOX"
    WINDOW "Batch 135: CheckBox", 200, 200, 500, 400 TO hWnd
    CONTROL ADD CHECKBOX, hWnd, 101, "Option A", 20, 20, 400, 30 TO hChk
    CONTROL ADD CHECKBOX, hWnd, 102, "Option B", 20, 60, 400, 30 TO hChk
    CONTROL ADD CHECKBOX, hWnd, 103, "Option C", 20, 100, 400, 30 TO hChk
    pb_message_loop
END FUNCTION


#ENDIF
