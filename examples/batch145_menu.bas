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

' Tier-3 DDT: MENU NEW BAR/POPUP/ADD STRING + DIALOG MENU
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hDlg AS QUAD
GLOBAL g_hMenuBar AS QUAD
GLOBAL g_hFileMenu AS QUAD
GLOBAL g_hHelpMenu AS QUAD

FUNCTION PBMAIN() AS LONG
    DIALOG NEW 0, "Batch 145: MENU + DIALOG", 100, 100, 400, 300 TO g_hDlg

    MENU NEW BAR TO g_hMenuBar
    MENU NEW POPUP TO g_hFileMenu
    MENU NEW POPUP TO g_hHelpMenu

    MENU ADD POPUP g_hMenuBar, g_hFileMenu, 0
    MENU ADD POPUP g_hMenuBar, g_hHelpMenu, 0

    MENU ADD STRING g_hFileMenu, "Open", 100, 0
    MENU ADD STRING g_hFileMenu, "Exit", 101, 0
    MENU ADD STRING g_hHelpMenu, "About", 102, 0

    DIALOG MENU g_hDlg, g_hMenuBar

    CONTROL ADD BUTTON, g_hDlg, 200, "Click Me", 150, 120, 100, 30
    DIALOG SHOW MODAL g_hDlg CALL DlgProc
END FUNCTION

CALLBACK FUNCTION DlgProc()
    SELECT CASE CB.MSG
        CASE 2
            DIALOG END g_hDlg, 1
        CASE 273
            IF CB.CTL = 200 THEN
                MSGBOX "Button clicked!"
            ELSEIF CB.CTL = 100 THEN
                MSGBOX "File -> Open clicked!"
            ELSEIF CB.CTL = 101 THEN
                DIALOG END g_hDlg, 1
            ELSEIF CB.CTL = 102 THEN
                MSGBOX "About: PB64Basic batch 145"
            END IF
    END SELECT
END FUNCTION


#ENDIF
