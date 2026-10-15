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

' Tier-3 DDT: BUTTON + EDITBOX - enable/disable controls
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)
FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hBtn AS QUAD
    LOCAL hEd AS QUAD
    MSGBOX "Batch 131: Tests CONTROL ADD EDITBOX - a text input field + OK button.", 0, "Test: EDITBOX"
    WINDOW "Batch 131: EditBox", 100, 100, 400, 300 TO hWnd
    CONTROL ADD EDITBOX, hWnd, 101, "Type here...", 20, 20, 200, 25 TO hEd
    CONTROL ADD BUTTON, hWnd, 102, "OK", 20, 60, 80, 30 TO hBtn
    pb_message_loop
END FUNCTION


#ENDIF
