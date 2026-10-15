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

' Tier-3 DDT: CONTROL GET TEXT simple test
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg AS QUAD
GLOBAL g_hEdit AS QUAD
GLOBAL hBtn AS QUAD
GLOBAL s AS STRING

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Test", 100, 100, 350, 250 TO g_hDlg
    CONTROL ADD EDITBOX, g_hDlg, 301, "Hello", 50, 50, 200, 25 TO g_hEdit
    CONTROL ADD BUTTON, g_hDlg, 302, "Read", 50, 100, 100, 30 TO hBtn
    DIALOG SHOW MODAL g_hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    SELECT CASE CB.MSG
        CASE 2
            DIALOG END g_hDlg, 1
        CASE 273
            IF CB.CTL = 302 THEN
                CONTROL GET TEXT g_hDlg, 301 TO s
                MSGBOX s
            END IF
    END SELECT
END FUNCTION


#ENDIF
