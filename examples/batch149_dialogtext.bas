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

' Tier-3 DDT: DIALOG SET TEXT - change window title at runtime
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg AS QUAD

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Batch 149: Dialog Before", 100, 100, 350, 200 TO g_hDlg
    CONTROL ADD BUTTON, g_hDlg, 100, "Change Title", 100, 70, 100, 30
    DIALOG SHOW MODAL g_hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    SELECT CASE CB.MSG
        CASE 2
            DIALOG END g_hDlg, 1
        CASE 273
            IF CB.CTL = 100 THEN
                DIALOG SET TEXT g_hDlg, "Dialog After"
                CONTROL SET TEXT g_hDlg, 100, "Changed!"
            END IF
    END SELECT
END FUNCTION


#ENDIF
