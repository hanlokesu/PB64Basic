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

FUNCTION COMPILE_ONLY() AS LONG
    LOCAL n AS LONG
    LOCAL s AS STRING
    CON.INPUT "value", n, s
    CON.LINE.INPUT "line", s
    CON.STDIN.LINE TO s
    INPUT "legacy", s
    CON.SCROLL.DOWN 1
    CON.SCROLL.LEFT 1
    CON.SCROLL.RIGHT 1
    CURSOR = 25
    CON.WAITSTAT
    CON.NEW
    CON.END
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL a AS LONG, b AS LONG, r AS LONG, c AS LONG, h AS QUAD
    LOCAL s AS STRING, s2 AS STRING
    LOCAL has_con AS LONG

    PRINT "batch377 console family"
    PRINT "A"; "B"
    ? "question form"
    CON.PRINT "con.print form"
    CLS
    CON.CLS

    COLOR 7, 0
    CON.COLOR 7, 0
    LOCATE 2, 3
    CON.LOCATE 2, 3

    h = CON.HANDLE
    has_con = (h <> 0)
    IF has_con THEN
        CON.CELL = 3, 5
        CON.CELL TO r, c
        CHECK r = 3, "CON.CELL row"
        CHECK c = 5, "CON.CELL col"
        a = CON.CELL.ROW
        CHECK a = 3, "CON.CELL.ROW"
        a = CON.CELL.COL
        CHECK a = 5, "CON.CELL.COL"
        a = CURSORX
        b = CURSORY
        CHECK a = 5, "CURSORX"
        CHECK b = 3, "CURSORY"
        CON.CURSOR = 25
        a = CON.CURSOR
        CHECK a = 25, "CON.CURSOR set/get"
        CON.CURSOR.OFF
        a = CON.CURSOR
        CHECK a = 0, "CON.CURSOR.OFF"
        CON.CURSOR.ON
        a = CON.CURSOR
        CHECK a = 25, "CON.CURSOR.ON restores"
    END IF

    PAGE 2
    IF has_con THEN
        CHECK PAGEACTIVE = 2, "PAGE n sets active only"
        CHECK PAGEVISIBLE = 1, "PAGE n leaves visible"
    END IF
    PAGE , 3
    IF has_con THEN
        CHECK PAGEACTIVE = 2, "PAGE , n leaves active"
        CHECK PAGEVISIBLE = 3, "PAGE , n sets visible"
    END IF
    PAGE 4, 5
    IF has_con THEN
        CHECK PAGEACTIVE = 4, "PAGE n,m active"
        CHECK PAGEVISIBLE = 5, "PAGE n,m visible"
    END IF
    PAGE 1, 1
    CON.PAGE.ACTIVE = 2
    CON.PAGE.VISIBLE = 1
    IF has_con THEN
        CHECK CON.PAGE.ACTIVE = 2, "CON.PAGE.ACTIVE set/get"
        CHECK CON.PAGE.VISIBLE = 1, "CON.PAGE.VISIBLE set/get"
    END IF
    PCOPY 1, 2
    CON.PCOPY 1, 3
    CON.PCOPY 9, 3
    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1
    IF has_con THEN
        CHECK PAGEACTIVE = 1, "PCOPY leaves active"
        CHECK PAGEVISIBLE = 1, "PCOPY leaves visible"
    END IF

    IF has_con THEN
        CON.SCREEN 30, 120
        CON.SCREEN TO r, c
        CHECK r = 30, "CON.SCREEN rows"
        CHECK c = 120, "CON.SCREEN cols"
        a = CON.SCREEN.ROW
        b = CON.SCREEN.COL
        CHECK a = 30, "CON.SCREEN.ROW"
        CHECK b = 120, "CON.SCREEN.COL"
        CON.VIRTUAL 60, 120
        CON.VIRTUAL TO r, c
        CHECK r = 60, "CON.VIRTUAL rows"
        CHECK c = 120, "CON.VIRTUAL cols"
        a = CON.VIRTUAL.ROW
        b = CON.VIRTUAL.COL
        CHECK a = 60, "CON.VIRTUAL.ROW"
        CHECK b = 120, "CON.VIRTUAL.COL"
        CHECK SCREENX = 120, "SCREENX virtual columns"
        CHECK SCREENY = 60, "SCREENY virtual rows"
        CON.VIEW 1, 1
        CON.VIEW TO r, c
        a = CON.VIEW.ROW
        b = CON.VIEW.COL
        CHECK a >= 1, "CON.VIEW.ROW"
        CHECK b >= 1, "CON.VIEW.COL"
        CON.SIZE TO r, c
        CHECK r > 0, "CON.SIZE.X positive"
        CHECK c > 0, "CON.SIZE.Y positive"
        a = CON.SIZE.X
        b = CON.SIZE.Y
        CHECK a > 0, "CON.SIZE.X getter"
        CHECK b > 0, "CON.SIZE.Y getter"
        CON.LOC TO r, c
        a = CON.LOC.X
        b = CON.LOC.Y

        CON.SCREEN 0, 120
        CHECK ERR = 5, "CON.SCREEN illegal value ERR 5"
        CON.SCREEN 30, 120
        CHECK ERR = 0, "CON.SCREEN success clears ERR"
        CON.VIRTUAL 10, 120
        CHECK ERR = 5, "CON.VIRTUAL smaller ERR 5"
        CON.VIRTUAL 60, 120
        CHECK ERR = 0, "CON.VIRTUAL success clears ERR"

        CONSOLE GET SCREEN TO r, c
        CHECK r = 30, "CONSOLE GET SCREEN"
        CONSOLE GET SIZE TO r, c
        CONSOLE GET VIEW TO r, c
        CONSOLE GET VIRTUAL TO r, c
        CONSOLE GET LOC TO r, c
        CONSOLE SET SCREEN 30, 120
        CONSOLE SET SIZE 640, 400
        CONSOLE SET VIEW 1, 1
        CONSOLE SET VIRTUAL 60, 120
        CONSOLE SET LOC 100, 50
        SCREEN 30, 120
        a = SCREENATTR(1, 1)
        a = SCREENX
        b = SCREENY

        CONSOLE NAME "batch377"
        s = CON.CAPTION$
        CHECK s = "batch377", "CON.CAPTION$"
        s = CONSNAME$
        CHECK s = "batch377", "CONSNAME$"
        CON.FOCUS
        CONSOLE SET FOCUS
    END IF

    DESKTOP GET CLIENT TO r, c
    DESKTOP GET LOC TO r, c
    DESKTOP GET SIZE TO r, c
    IF has_con THEN
        CHECK r > 0, "DESKTOP GET SIZE rows"
        CHECK c > 0, "DESKTOP GET SIZE cols"
    END IF

    CON.CLS
    CON.COLOR 15, 1
    CON.CELL = 1, 1
    CON.PRINT "X" ;
    IF has_con THEN
        a = CON.SCREEN.ATTR(1, 1)
        CHECK a = 31, "CON.COLOR attribute"
        a = SCREENATTR(1, 1)
        CHECK a = 31, "SCREENATTR"
        a = CON.SCREEN.CHAR(1, 1)
        CHECK a = 88, "CON.SCREEN.CHAR"
    END IF

    IF has_con THEN
        CON.CLS
        CON.CELL = 30, 1
        CON.PRINT "A"
        a = CON.SCREEN.CHAR(29, 1)
        b = CON.SCREEN.CHAR(30, 1)
        CHECK a = 65, "CON.PRINT bottom scroll moves A up"
        CHECK b = 32, "CON.PRINT bottom scroll blanks last row"
        CON.CELL = 30, 1
        CON.PRINT "B"
        a = CON.SCREEN.CHAR(29, 1)
        CHECK a = 66, "CON.PRINT second bottom scroll"

        CON.CLS
        CON.CELL = 1, 1
        CON.PRINT "A"
        CON.CELL = 2, 1
        CON.PRINT "B"
        CON.CELL = 3, 1
        CON.PRINT "C"
        CON.SCROLL.UP 1
        a = CON.SCREEN.CHAR(1, 1)
        b = CON.SCREEN.CHAR(2, 1)
        CHECK a = 32, "CON.SCROLL.UP blanks top"
        CHECK b = 65, "CON.SCROLL.UP moves A down"
        SCROLL DOWN, 1
    END IF

    a = CON.IN
    a = CON.OUT
    a = CON.STDEOF
    a = CONSOUT
    a = CON.INSHIFT
    a = CON.INSTAT
    s = CON.INKEY$
    s = CON.WAITKEY$("", 0)
    CON.INPUT.FLUSH
    CON.KEY 30, "F11"
    CON.KEY 31, "F12"
    s = CON.KEY$(30)
    IF has_con THEN
        CHECK s = "F11", "CON.KEY 30 alias"
    END IF
    s = CON.KEY$(31)

    CON.MOUSE 15, 31
    CON.MOUSE.ON
    a = CON.MOUSE.BUTTONS
    a = CON.MOUSE.COL
    b = CON.MOUSE.ROW
    a = CON.MOUSE.STAT
    CON.MOUSE.OFF

    CON.STDOUT "stdout" ;
    CON.STDERR "stderr" + CHR$(13, 10)
    STDOUT "legacy stdout" ;
    STDERR "legacy stderr" + CHR$(13, 10)
    h = GETSTDOUT
    h = GETSTDERR
    h = GETSTDIN
    h = GETSTDVID(0)

    PRINT "batch377 done, failures="; g_fails
    FUNCTION = g_fails
END FUNCTION