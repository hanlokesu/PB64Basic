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

' Tier-3 DDT: BUTTON + EDITBOX - show/hide/enable/disable
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hBtn1 AS QUAD
    LOCAL hBtn2 AS QUAD
    LOCAL hBtn3 AS QUAD
    LOCAL hBtn4 AS QUAD
    LOCAL hEdit AS QUAD

    WINDOW "Batch 138: CONTROL SHOW/HIDE/ENABLE/FOCUS", 100, 100, 400, 250 TO hWnd

    CONTROL ADD BUTTON, hWnd, 101, "Show Edit",  20, 20, 100, 25 TO hBtn1
    CONTROL ADD BUTTON, hWnd, 102, "Hide Edit",  20, 55, 100, 25 TO hBtn2
    CONTROL ADD BUTTON, hWnd, 103, "Disable",    20, 90, 100, 25 TO hBtn3
    CONTROL ADD BUTTON, hWnd, 104, "Enable",     20, 125, 100, 25 TO hBtn4
    CONTROL ADD EDITBOX, hWnd, 105, "hello", 150, 20, 200, 25 TO hEdit

    ' Test HIDE: hide the edit box
    CONTROL HIDE hEdit
    MSGBOX "Edit box is now HIDDEN. Click OK to show it.", 0, "Test HIDE"

    ' Test SHOW: show it again
    CONTROL SHOW hEdit
    MSGBOX "Edit box is now SHOWN. Click OK to disable it.", 0, "Test SHOW"

    ' Test DISABLE
    CONTROL DISABLE hEdit
    MSGBOX "Edit box is now DISABLED (greyed out). Click OK to enable it.", 0, "Test DISABLE"

    ' Test ENABLE
    CONTROL ENABLE hEdit
    MSGBOX "Edit box is now ENABLED again. Click OK to give it focus.", 0, "Test ENABLE"

    ' Test FOCUS
    CONTROL FOCUS hEdit
    MSGBOX "Edit box has FOCUS (you should see the cursor in it). Click OK to finish.", 0, "Test FOCUS"

    MSGBOX "All 5 tests done! (SHOW/HIDE/ENABLE/DISABLE/FOCUS)", 0, "Batch 138"
END FUNCTION


#ENDIF
