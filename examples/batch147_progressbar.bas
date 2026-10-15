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

' Tier-3 DDT: CONTROL ADD PROGRESSBAR
' Click button to update label
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg AS QUAD
GLOBAL hProg AS QUAD
GLOBAL hBtn AS QUAD
GLOBAL hLabel AS QUAD

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Batch 147: PROGRESSBAR", 100, 100, 400, 200 TO g_hDlg
    
    CONTROL ADD LABEL, g_hDlg, 101, "Progress bar shown below:", 20, 20, 200, 12 TO hLabel
    CONTROL ADD PROGRESSBAR, g_hDlg, 102, 20, 40, 340, 20 TO hProg
    CONTROL ADD BUTTON, g_hDlg, 103, "Click Me", 130, 80, 140, 30 TO hBtn
    
    DIALOG SHOW MODAL g_hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    SELECT CASE CB.MSG
        CASE 2
            DIALOG END g_hDlg, 1
        CASE 273
            IF CB.CTL = 103 THEN
                MSGBOX "Button clicked! Progress bar is visible above."
            END IF
    END SELECT
END FUNCTION


#ENDIF
