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

' Tier-3 DDT: ALL GUI controls combined demo
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hEdit AS QUAD
GLOBAL g_hChk AS QUAD
GLOBAL g_hRadio AS QUAD
GLOBAL g_hVSB AS QUAD
GLOBAL g_hHSB AS QUAD

FUNCTION OnShowState
    LOCAL s AS STRING
    LOCAL chk AS LONG
    LOCAL rad AS LONG

    CONTROL GET TEXT g_hEdit TO s$
    CONTROL GET CHECK g_hChk TO chk
    CONTROL GET CHECK g_hRadio TO rad

    MSGBOX "Edit: [" + s$ + "]" + $CRLF + _
           "Checkbox: " + STR$(chk) + $CRLF + _
           "Radio B: " + STR$(rad), 0, "State"
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD
    LOCAL hBtn AS QUAD

    WINDOW "GUI Test", 100, 100, 700, 500 TO hWnd

    CONTROL ADD BUTTON, hWnd, 101, "Show State", 30, 20, 150, 35 TO hBtn
    CONTROL CMD hBtn, OnShowState

    CONTROL ADD EDITBOX, hWnd, 102, "Type here", 30, 70, 300, 28 TO g_hEdit
    CONTROL ADD CHECKBOX, hWnd, 103, "Option A (checkbox)", 30, 115, 300, 28 TO g_hChk
    CONTROL ADD RADIOBUTTON, hWnd, 104, "Choice B (radiobutton)", 30, 155, 300, 28 TO g_hRadio
    CONTROL ADD SCROLLBAR, hWnd, 106, 600, 20, 30, 300 TO g_hVSB
    CONTROL ADD HSCROLLBAR, hWnd, 107, 30, 420, 560, 30 TO g_hHSB

    MSGBOX "Ready! Try scrollbars many times", 0, "Test"

    MESSAGE LOOP
END FUNCTION


#ENDIF
