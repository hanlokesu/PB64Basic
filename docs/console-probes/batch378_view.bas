' batch378_view - diagnostic for CON.VIEW / CONSOLE SET VIEW on this host.
' The runtime calls SetConsoleWindowInfo(TRUE, absolute rect), which returns
' success here, but the console viewport never moves.  This probe records the
' exact screen/virtual/view values instead of pretending the assertion passed.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL r AS LONG, c AS LONG, vr AS LONG, vc AS LONG
    LOCAL er AS LONG

    OPEN "batch378_view.txt" FOR OUTPUT AS #1

    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1
    CON.SCREEN 10, 80
    er = ERR
    CON.SCREEN TO r, c
    PRINT #1, "screen=" + STR$(r) + "," + STR$(c) + " err=" + STR$(er)

    CON.VIRTUAL 40, 120
    er = ERR
    CON.VIRTUAL TO r, c
    PRINT #1, "virtual=" + STR$(r) + "," + STR$(c) + " err=" + STR$(er)

    CON.VIEW 5, 7
    er = ERR
    vr = CON.VIEW.ROW
    vc = CON.VIEW.COL
    PRINT #1, "after CON.VIEW 5,7: row=" + STR$(vr) + " col=" + STR$(vc) + " err=" + STR$(er)

    CONSOLE GET VIEW TO vr, vc
    PRINT #1, "CONSOLE GET VIEW: row=" + STR$(vr) + " col=" + STR$(vc)

    CONSOLE SET VIEW 3, 4
    er = ERR
    CONSOLE GET VIEW TO vr, vc
    PRINT #1, "after CONSOLE SET VIEW 3,4: row=" + STR$(vr) + " col=" + STR$(vc) + " err=" + STR$(er)

    CON.SCREEN 30, 120
    CLOSE #1
END FUNCTION
