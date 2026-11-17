#COMPILE EXE
#CONSOLE ON
#DIM ALL
GLOBAL g_fails AS LONG

FUNCTION CHECK(cond AS LONG, label AS STRING) AS LONG
    IF cond = 0 THEN
        CON.STDERR "FAIL: " + label + CHR$(13, 10)
        g_fails = g_fails + 1
    END IF
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL a AS LONG, r AS LONG, c AS LONG, h AS QUAD
    LOCAL s AS STRING

    PRINT "batch376 console family"
    PRINT "A"; "B", "C"
    ? "question form"
    CON.PRINT "con.print form"
    CLS
    CON.CLS

    COLOR 7, 0
    CON.COLOR 7, 0
    LOCATE 2, 3
    CON.LOCATE 2, 3
    CON.CELL = 3, 5
    CON.CELL TO r, c
    CHECK r = 3, "CON.CELL row"
    CHECK c = 5, "CON.CELL col"
    a = CON.CELL.ROW
    CHECK a = 3, "CON.CELL.ROW"
    a = CON.CELL.COL
    CHECK a = 5, "CON.CELL.COL"
    a = CURSORX
    a = CURSORY
    CON.CURSOR = 25
    a = CON.CURSOR
    CHECK a = 25, "CON.CURSOR"
    CON.CURSOR.ON 25
    CON.CURSOR.OFF

    PAGE 2, 1
    CHECK PAGEACTIVE = 2, "PAGEACTIVE"
    CHECK PAGEVISIBLE = 1, "PAGEVISIBLE"
    PCOPY 1, 2
    a = CON.PAGE.ACTIVE
    CHECK a = 2, "CON.PAGE.ACTIVE"
    a = CON.PAGE.VISIBLE
    CHECK a = 1, "CON.PAGE.VISIBLE"
    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1
    CON.PCOPY 1, 3
    SCROLL DOWN, 1
    SCROLL UP
    CON.SCROLL.LEFT 1
    CON.SCROLL.RIGHT 1

    CON.SCREEN 25, 80
    CON.SCREEN TO r, c
    a = CON.SCREEN.ROW
    a = CON.SCREEN.COL
    a = CON.SCREEN.ATTR(1, 1)
    a = CON.SCREEN.CHAR(1, 1)
    CON.VIRTUAL 30, 90
    CON.VIRTUAL TO r, c
    a = CON.VIRTUAL.ROW
    a = CON.VIRTUAL.COL
    CON.VIEW 5, 10
    CON.VIEW TO r, c
    a = CON.VIEW.ROW
    a = CON.VIEW.COL
    CON.SIZE 640, 400
    CON.SIZE TO r, c
    a = CON.SIZE.X
    a = CON.SIZE.Y
    CON.LOC 100, 50
    CON.LOC TO r, c
    a = CON.LOC.X
    a = CON.LOC.Y

    CONSOLE GET SCREEN TO r, c
    CONSOLE GET SIZE TO r, c
    CONSOLE GET VIEW TO r, c
    CONSOLE GET VIRTUAL TO r, c
    CONSOLE GET LOC TO r, c
    CONSOLE SET SCREEN 25, 80
    CONSOLE SET SIZE 640, 400
    CONSOLE SET VIEW 5, 10
    CONSOLE SET VIRTUAL 30, 90
    CONSOLE SET LOC 100, 50
    SCREEN 25, 80
    a = SCREENATTR(1, 1)
    a = SCREENX
    a = SCREENY

    CONSOLE NAME "batch376"
    s = CON.CAPTION$
    CHECK s = "batch376", "CON.CAPTION$"
    s = CONSNAME$
    CHECK s = "batch376", "CONSNAME$"
    CON.FOCUS
    CONSOLE SET FOCUS
    CON.HANDLE TO h
    a = CON.IN
    a = CON.OUT
    a = CON.STDEOF
    a = CONSOUT
    CON.STDOUT "stdout"
    CON.STDERR "stderr"
    CON.INPUT.FLUSH
    a = CON.INSHIFT
    a = CON.INSTAT
    s = CON.INKEY$
    s = CON.WAITKEY$("", 0)
    CON.KEY 1, "F1"
    s = CON.KEY$(1)
    CHECK s = "F1", "CON.KEY$"
    CON.MOUSE 1, 1
    CON.MOUSE.ON
    a = CON.MOUSE.BUTTONS
    a = CON.MOUSE.COL
    a = CON.MOUSE.ROW
    a = CON.MOUSE.STAT
    CON.MOUSE.OFF

    PRINT "batch376 done, failures="; g_fails
    FUNCTION = g_fails
END FUNCTION
