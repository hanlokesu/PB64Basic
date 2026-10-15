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

' Tier-3 DDT: EDITBOX basic typing test
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


CALLBACK FUNCTION DlgProc()
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS QUAD
    LOCAL hEdit AS QUAD
    DIALOG NEW 0, "Edit Test", , , 200, 100 TO hDlg
    CONTROL ADD EDITBOX, hDlg, 1001, "", 10, 10, 180, 20 TO hEdit
    DIALOG SHOW MODAL hDlg CALL DlgProc
END FUNCTION


#ENDIF
