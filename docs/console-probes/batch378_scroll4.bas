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

FUNCTION PUTCH(r AS LONG, c AS LONG, ch AS STRING) AS LONG
    CON.CELL = r, c
    CON.PRINT ch ;
    FUNCTION = 0
END FUNCTION

FUNCTION CHKCH(r AS LONG, c AS LONG, ch AS STRING, label AS STRING) AS LONG
    LOCAL got AS LONG
    got = CON.SCREEN.CHAR(r, c)
    CHECK got = ASC(ch), label + " got=" + STR$(got) + " want=" + STR$(ASC(ch))
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL h AS QUAD
    LOCAL allocated AS LONG

    h = CON.HANDLE
    IF h = 0 THEN
        CON.NEW
        allocated = 1
        h = CON.HANDLE
    END IF
    IF h = 0 THEN
        CON.STDERR "batch378 scroll4 SKIP: no console handle" + CHR$(13, 10)
        FUNCTION = 0
        EXIT FUNCTION
    END IF

    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1

    ' DOWN: rows 1..4 should become B/C/blank/blank (content moves up).
    CON.CLS
    PUTCH 1, 1, "A"
    PUTCH 2, 1, "B"
    PUTCH 3, 1, "C"
    CON.SCROLL.DOWN 1
    CHKCH 1, 1, "B", "SCROLL.DOWN row1 B"
    CHKCH 2, 1, "C", "SCROLL.DOWN row2 C"
    CHKCH 3, 1, " ", "SCROLL.DOWN row3 blank"
    CHKCH 4, 1, " ", "SCROLL.DOWN row4 blank"

    ' LEFT: row 1 should become blank/A/B (content moves right).
    CON.CLS
    PUTCH 1, 1, "A"
    PUTCH 1, 2, "B"
    PUTCH 1, 3, "C"
    CON.SCROLL.LEFT 1
    CHKCH 1, 1, " ", "SCROLL.LEFT col1 blank"
    CHKCH 1, 2, "A", "SCROLL.LEFT col2 A"
    CHKCH 1, 3, "B", "SCROLL.LEFT col3 B"

    ' RIGHT: row 1 should become B/C/blank (content moves left).
    CON.CLS
    PUTCH 1, 1, "A"
    PUTCH 1, 2, "B"
    PUTCH 1, 3, "C"
    CON.SCROLL.RIGHT 1
    CHKCH 1, 1, "B", "SCROLL.RIGHT col1 B"
    CHKCH 1, 2, "C", "SCROLL.RIGHT col2 C"
    CHKCH 1, 3, " ", "SCROLL.RIGHT col3 blank"
    CHKCH 1, 4, " ", "SCROLL.RIGHT col4 blank"

    ' Oversized DOWN clears the whole rectangle.
    CON.CLS
    PUTCH 1, 1, "A"
    PUTCH 2, 1, "B"
    PUTCH 3, 1, "C"
    CON.SCROLL.DOWN 99
    CHKCH 1, 1, " ", "SCROLL.DOWN 99 row1 blank"
    CHKCH 2, 1, " ", "SCROLL.DOWN 99 row2 blank"
    CHKCH 3, 1, " ", "SCROLL.DOWN 99 row3 blank"

    CON.CLS
    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1
    CON.STDERR "batch378 scroll4, failures=" + STR$(g_fails) + CHR$(13, 10)
    IF allocated THEN
        CON.END
    END IF
    FUNCTION = g_fails
END FUNCTION
