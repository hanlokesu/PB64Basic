' batch378_exact - precise console assertions for CLS / CON.CLS, LOCATE,
' PCOPY content, CON.SIZE / CON.VIEW / CON.LOC, and CONSOLE GET/SET.
' Run only in a real console window (Git CMD / conhost).
#COMPILE EXE
#CONSOLE ON
#DIM ALL

GLOBAL g_fails AS LONG

FUNCTION CHECK(cond AS LONG, label AS STRING) AS LONG
    IF cond = 0 THEN
        PRINT #1, "FAIL: " + label
        g_fails = g_fails + 1
    END IF
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL a AS LONG, b AS LONG, r AS LONG, c AS LONG, x AS LONG, y AS LONG
    LOCAL w AS LONG, h AS LONG
    LOCAL szx AS LONG, szy AS LONG, lx AS LONG, ly AS LONG
    LOCAL vx AS LONG, vy AS LONG

    OPEN "batch378_exact.txt" FOR OUTPUT AS #1
    CON.STDERR "exact: open err=" + STR$(ERR) + CHR$(13, 10)

    ' --- CLS clears the active page -------------------------------------
    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1
    CON.CLS
    CON.CELL = 1, 1
    CON.PRINT "A" ;
    CON.CELL = 10, 80
    CON.PRINT "B" ;
    CLS
    a = CON.SCREEN.CHAR(1, 1)
    b = CON.SCREEN.CHAR(10, 80)
    CHECK a = 32, "CLS row1 col1 blank"
    CHECK b = 32, "CLS row10 col80 blank"

    CON.CLS
    CON.CELL = 1, 1
    CON.PRINT "A" ;
    CON.CELL = 10, 80
    CON.PRINT "B" ;
    CON.CLS
    a = CON.SCREEN.CHAR(1, 1)
    b = CON.SCREEN.CHAR(10, 80)
    CHECK a = 32, "CON.CLS row1 col1 blank"
    CHECK b = 32, "CON.CLS row10 col80 blank"
    CON.STDERR "exact: cls done" + CHR$(13, 10)

    ' --- LOCATE and CON.LOCATE set the cursor exactly --------------------
    LOCATE 10, 20
    a = CURSORX
    b = CURSORY
    CHECK a = 20, "LOCATE col"
    CHECK b = 10, "LOCATE row"
    CON.LOCATE 7, 9
    a = CURSORX
    b = CURSORY
    CHECK a = 9, "CON.LOCATE col"
    CHECK b = 7, "CON.LOCATE row"

    ' --- PCOPY copies page content, not just the page number ------------
    CON.PAGE.ACTIVE = 1
    CON.CLS
    CON.CELL = 1, 1
    CON.PRINT "A" ;
    CON.CELL = 2, 1
    CON.PRINT "B" ;
    PCOPY 1, 2
    CON.PAGE.ACTIVE = 2
    a = CON.SCREEN.CHAR(1, 1)
    b = CON.SCREEN.CHAR(2, 1)
    CHECK a = 65, "PCOPY 1,2 row1 A"
    CHECK b = 66, "PCOPY 1,2 row2 B"

    CON.PCOPY 1, 3
    CON.PAGE.ACTIVE = 3
    a = CON.SCREEN.CHAR(1, 1)
    b = CON.SCREEN.CHAR(2, 1)
    CHECK a = 65, "CON.PCOPY 1,3 row1 A"
    CHECK b = 66, "CON.PCOPY 1,3 row2 B"

    CON.CELL = 1, 1
    CON.PRINT "Z" ;
    CON.PCOPY 9, 3
    CON.PAGE.ACTIVE = 3
    a = CON.SCREEN.CHAR(1, 1)
    CHECK a = 90, "CON.PCOPY 9,3 invalid leaves page3 Z"
    CHECK ERR = 0, "CON.PCOPY invalid keeps ERR 0"

    ' --- CON.SIZE / CONSOLE GET SIZE are the same values ----------------
    CON.SIZE TO w, h
    a = CON.SIZE.X
    b = CON.SIZE.Y
    CHECK a = w, "CON.SIZE.X matches CON.SIZE TO wide"
    CHECK b = h, "CON.SIZE.Y matches CON.SIZE TO high"
    CONSOLE GET SIZE TO x, y
    CHECK x = a, "CONSOLE GET SIZE wide"
    CHECK y = b, "CONSOLE GET SIZE high"

    ' CON.VIEW is covered by batch378_view.bas: this host accepts
    ' SetConsoleWindowInfo but does not move the viewport, so an exact
    ' assertion here would only record an environment limitation.

    ' --- CON.LOC / CONSOLE GET LOC round-trip without moving the window ---
    CON.LOC TO lx, ly
    a = CON.LOC.X
    b = CON.LOC.Y
    CHECK a = lx, "CON.LOC.X matches CON.LOC TO x"
    CHECK b = ly, "CON.LOC.Y matches CON.LOC TO y"
    CONSOLE GET LOC TO x, y
    CHECK x = a, "CONSOLE GET LOC x"
    CHECK y = b, "CONSOLE GET LOC y"
    CONSOLE SET LOC lx, ly
    CONSOLE GET LOC TO x, y
    CHECK x = lx, "CONSOLE SET LOC same x"
    CHECK y = ly, "CONSOLE SET LOC same y"

    ' --- CON.SIZE setter round-trip with the current size ----------------
    CON.SIZE TO szx, szy
    CONSOLE SET SIZE szx, szy
    CONSOLE GET SIZE TO x, y
    CHECK x = szx, "CONSOLE SET SIZE same wide"
    CHECK y = szy, "CONSOLE SET SIZE same high"

    ' Restore the default working screen for later probes.
    CON.SCREEN 30, 120
    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1
    CON.CLS

    PRINT #1, "batch378 exact, failures= " + STR$(g_fails)
    CLOSE #1
END FUNCTION
