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

' Tier-3 DDT: CONTROL HIDE / SHOW / ENABLE / DISABLE
' Buttons to toggle a target control
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg AS QUAD
GLOBAL hTarget AS QUAD
GLOBAL hHideBtn AS QUAD
GLOBAL hShowBtn AS QUAD
GLOBAL hDisBtn AS QUAD
GLOBAL hEnBtn AS QUAD

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Batch 152: CONTROL STATE", 100, 100, 350, 250 TO g_hDlg
    DIALOG CENTER g_hDlg
    CONTROL ADD BUTTON, g_hDlg, 101, "Target Button", 125, 20, 100, 30 TO hTarget
    CONTROL ADD BUTTON, g_hDlg, 102, "Hide", 30, 80, 60, 25 TO hHideBtn
    CONTROL ADD BUTTON, g_hDlg, 103, "Show", 100, 80, 60, 25 TO hShowBtn
    CONTROL ADD BUTTON, g_hDlg, 104, "Disable", 170, 80, 60, 25 TO hDisBtn
    CONTROL ADD BUTTON, g_hDlg, 105, "Enable", 240, 80, 60, 25 TO hEnBtn
    DIALOG SHOW MODAL g_hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    SELECT CASE CB.MSG
        CASE 2
            DIALOG END g_hDlg, 1
        CASE 273
            SELECT CASE CB.CTL
                CASE 102: CONTROL HIDE hTarget
                CASE 103: CONTROL SHOW hTarget
                CASE 104: CONTROL DISABLE hTarget
                CASE 105: CONTROL ENABLE hTarget
            END SELECT
    END SELECT
END FUNCTION


#ENDIF
