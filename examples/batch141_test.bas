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

' Tier-3 DDT: SCROLLBAR - vertical/horizontal scrollbar
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hScroll AS QUAD
    LOCAL pos AS LONG

    WINDOW "Batch 141: SCROLLBAR", 100, 100, 300, 300 TO hWnd

    CONTROL ADD SCROLLBAR, hWnd, 301, 250, 30, 25, 200 TO hScroll

    ' Set position to 50
    CONTROL SET POS hScroll, 50
    CONTROL GET POS hScroll TO pos
    MSGBOX "Scrollbar position set to 50, read back = " + STR$(pos), 0, "Batch 141"

    ' Set to 100
    CONTROL SET POS hScroll, 100
    CONTROL GET POS hScroll TO pos
    MSGBOX "Position now = " + STR$(pos), 0, "Batch 141"

    MSGBOX "Done!", 0, "Batch 141"
END FUNCTION


#ENDIF
