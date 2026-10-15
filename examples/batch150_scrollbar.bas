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

' Tier-3 DDT: SCROLLBAR (vertical)
' Drag scrollbar, click button to show value
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg AS QUAD
GLOBAL hVScroll AS QUAD
GLOBAL hBtn AS QUAD
GLOBAL hLabel AS QUAD

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Batch 150: SCROLLBAR", 100, 100, 300, 250 TO g_hDlg
    DIALOG CENTER g_hDlg
    CONTROL ADD LABEL, g_hDlg, 101, "Drag the vertical scrollbar:", 20, 20, 200, 12 TO hLabel
    CONTROL ADD SCROLLBAR, g_hDlg, 102, 20, 40, 20, 150 TO hVScroll
    CONTROL ADD BUTTON, g_hDlg, 103, "Show Value", 60, 100, 100, 30 TO hBtn
    DIALOG SHOW MODAL g_hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    SELECT CASE CB.MSG
        CASE 2
            DIALOG END g_hDlg, 1
        CASE 273
            IF CB.CTL = 103 THEN
                MSGBOX "Scrollbar value"
            END IF
    END SELECT
END FUNCTION


#ENDIF
