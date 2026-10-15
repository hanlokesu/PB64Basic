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

' Tier-3 DDT: CONTROL ADD EDITBOX - text input
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)
FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hEd AS QUAD
    LOCAL hBtn AS QUAD
    LOCAL s AS STRING
    MSGBOX "Batch 132: Tests CONTROL GET/SET TEXT - reads and writes editbox text.", 0, "Test: GET/SET TEXT"
    WINDOW "Batch 132: GET/SET TEXT", 100, 100, 400, 200 TO hWnd
    CONTROL ADD EDITBOX, hWnd, 101, "Hello", 20, 20, 200, 25 TO hEd
    CONTROL GET TEXT hEd TO s
    MSGBOX "Initial text: " + s, 0, "GET TEXT"
    CONTROL SET TEXT hEd, "Changed!"
    CONTROL GET TEXT hEd TO s
    MSGBOX "After SET: " + s, 0, "SET TEXT"
    pb_message_loop
END FUNCTION


#ENDIF
