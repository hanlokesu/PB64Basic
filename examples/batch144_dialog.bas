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

' Tier-3 DDT: DIALOG NEW + DIALOG SHOW MODAL
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg AS QUAD
GLOBAL g_hBtn AS QUAD

CALLBACK FUNCTION DlgProc()
    DIALOG END 0, 1
END FUNCTION

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Test", 100, 100, 300, 200 TO g_hDlg
    CONTROL ADD BUTTON, g_hDlg, 100, "OK", 100, 60, 80, 30 TO g_hBtn
    DIALOG SHOW MODAL g_hDlg CALL DlgProc
END FUNCTION


#ENDIF
