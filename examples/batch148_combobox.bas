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

' Tier-3 DDT: COMBOBOX + LISTBOX interaction
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg AS QUAD
GLOBAL g_hCombo AS QUAD
GLOBAL g_hList AS QUAD

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Batch 148: Combo and List", 100, 100, 400, 350 TO g_hDlg
    CONTROL ADD COMBOBOX, g_hDlg, 201, "", 50, 50, 200, 100 TO g_hCombo
    CONTROL ADD LISTBOX, g_hDlg, 202, "", 50, 150, 200, 100 TO g_hList
    CONTROL ADD BUTTON, g_hDlg, 203, "Add to Combo", 280, 50, 100, 30
    CONTROL ADD BUTTON, g_hDlg, 204, "Close", 280, 150, 100, 30
    DIALOG SHOW MODAL g_hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    STATIC n AS LONG
    SELECT CASE CB.MSG
        CASE 2
            DIALOG END g_hDlg, 1
        CASE 273
            IF CB.CTL = 203 THEN
                n = n + 1
                COMBOBOX ADD g_hCombo, "Item " + STR$(n)
                LISTBOX ADD g_hList, "Item " + STR$(n)
            END IF
            IF CB.CTL = 204 THEN
                DIALOG END g_hDlg, 1
            END IF
    END SELECT
END FUNCTION


#ENDIF
