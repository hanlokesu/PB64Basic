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

' Tier-3 DDT: CONTROL ADD LISTBOX - list selection
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)
FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hLb AS QUAD
    MSGBOX "Batch 134: Tests CONTROL ADD LISTBOX - a list box control.", 0, "Test: LISTBOX"
    WINDOW "Batch 134: ListBox", 100, 100, 400, 300 TO hWnd
    CONTROL ADD LISTBOX, hWnd, 101, , 20, 20, 200, 200 TO hLb
    pb_message_loop
END FUNCTION


#ENDIF
