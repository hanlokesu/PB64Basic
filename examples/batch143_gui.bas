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

' Tier-3 DDT: LABEL + PROGRESSBAR
' Auto-generated: PB64Basic fork batch test
' #CONSOLE OFF (commented: PBWin10 rejects #CONSOLE; fork ignores it)


GLOBAL g_hLabel AS QUAD
GLOBAL g_hProg AS QUAD

FUNCTION PBMAIN() AS LONG
    LOCAL hWnd AS QUAD

    WINDOW "Label + Progress", 100, 100, 500, 300 TO hWnd

    CONTROL ADD LABEL, hWnd, 201, "Progress:", 30, 30, 100, 20 TO g_hLabel
    CONTROL ADD PROGRESSBAR, hWnd, 202, 30, 80, 400, 30 TO g_hProg

    MESSAGE LOOP
END FUNCTION


#ENDIF
