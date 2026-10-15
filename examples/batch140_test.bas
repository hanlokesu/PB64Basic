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

' Tier-3 DDT: BUTTON + CHECKBOX - control state change
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hChk1 AS QUAD
    LOCAL hChk2 AS QUAD
    LOCAL hBtn AS QUAD
    LOCAL result AS LONG

    WINDOW "Batch 140: CONTROL CHECK/UNCHECK/GET CHECK", 100, 100, 380, 280 TO hWnd

    CONTROL ADD CHECKBOX, hWnd, 201, "Option A",  30, 30, 200, 25 TO hChk1
    CONTROL ADD CHECKBOX, hWnd, 202, "Option B",  30, 70, 200, 25 TO hChk2
    CONTROL ADD BUTTON,   hWnd, 203, "Read State", 30, 130, 120, 35 TO hBtn

    ' Check both by default
    CONTROL CHECK hChk1
    CONTROL CHECK hChk2

    MSGBOX "Both checkboxes are CHECKED. Click OK to read their state.", 0, "Batch 140"

    CONTROL GET CHECK hChk1 TO result
    MSGBOX "Option A checked = " + STR$(result), 0, "GET CHECK"

    CONTROL GET CHECK hChk2 TO result
    MSGBOX "Option B checked = " + STR$(result), 0, "GET CHECK"

    ' Uncheck both
    CONTROL UNCHECK hChk1
    CONTROL UNCHECK hChk2
    MSGBOX "Both are now UNCHECKED.", 0, "Batch 140"

    CONTROL GET CHECK hChk1 TO result
    MSGBOX "Option A checked = " + STR$(result) + " (should be 0)", 0, "GET CHECK"

    MSGBOX "Done!", 0, "Batch 140"
END FUNCTION


#ENDIF
