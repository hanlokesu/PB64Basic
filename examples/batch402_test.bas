#COMPILE EXE
#CONSOLE ON
#DIM ALL

' batch 402: graphic metrics, font isolation, comma zones and WSTRING SafeArrays.

FUNCTION CheckFont(BYVAL face AS STRING, BYVAL points AS SINGLE, BYVAL want_w AS SINGLE, BYVAL want_h AS SINGLE, BYREF fails AS LONG) AS LONG
    LOCAL hf AS DWORD
    LOCAL x AS SINGLE, y AS SINGLE
    LOCAL cw AS SINGLE, ch AS SINGLE
    LOCAL bw AS SINGLE, bh AS SINGLE

    FONT NEW face, points, 0 TO hf
    GRAPHIC SET FONT hf
    GRAPHIC CELL SIZE TO cw, ch
    PRINT #1, face; " cell="; cw; ","; ch
    IF cw <> want_w OR ch <> want_h THEN INCR fails

    GRAPHIC SET POS (0, 0)
    GRAPHIC PRINT "A",
    GRAPHIC GET POS TO x, y
    PRINT #1, face; " zone="; x
    IF x <> 14 * cw THEN INCR fails

    GRAPHIC SET POS (0, 0)
    GRAPHIC TEXT SIZE "B" TO bw, bh
    GRAPHIC PRINT "A", "B";
    GRAPHIC GET POS TO x, y
    PRINT #1, face; " ab="; x
    IF x <> 14 * cw + bw THEN INCR fails

    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL hbmp AS DWORD
    LOCAL cw AS SINGLE, ch AS SINGLE, cw2 AS SINGLE, ch2 AS SINGLE
    LOCAL x AS SINGLE, y AS SINGLE
    LOCAL r AS LONG, c AS LONG
    LOCAL v AS VARIANT

    fails = 0
    OPEN "batch402_test_out.txt" FOR OUTPUT AS #1

    GRAPHIC BITMAP NEW 800, 220 TO hbmp
    GRAPHIC ATTACH hbmp, 0

    GRAPHIC CELL SIZE TO cw, ch
    PRINT #1, "default cell="; cw; ","; ch
    IF cw <> 5 OR ch <> 13 THEN INCR fails
    PRINT #1, "after default cell fails="; fails

    GRAPHIC CHR SIZE TO cw2, ch2
    PRINT #1, "default chr="; cw2; ","; ch2
    IF cw2 <> 5 OR ch2 <> 13 THEN INCR fails
    IF GRAPHIC(CHR.SIZE.X) <> 5 THEN INCR fails
    IF GRAPHIC(CHR.SIZE.Y) <> 13 THEN INCR fails
    PRINT #1, "after chr fails="; fails

    CheckFont "Arial", 12, 7, 19, fails
    CheckFont "Times New Roman", 12, 6, 20, fails
    CheckFont "Courier New", 12, 10, 18, fails

    GRAPHIC SET FONT 0
    GRAPHIC CELL SIZE TO cw, ch
    GRAPHIC CELL = 2, 3
    GRAPHIC GET POS TO x, y
    PRINT #1, "cell pos="; x; ","; y
    IF x <> 2 * cw OR y <> ch THEN INCR fails

    GRAPHIC CELL TO r, c
    PRINT #1, "cell read="; r; ","; c
    IF r <> 2 OR c <> 3 THEN INCR fails
    IF GRAPHIC(ROW) <> 2 OR GRAPHIC(COL) <> 3 THEN INCR fails

    GRAPHIC CELL = 0, 5
    GRAPHIC CELL TO r, c
    GRAPHIC GET POS TO x, y
    PRINT #1, "cell zero="; r; ","; c; " pos="; x; ","; y
    IF r <> 2 OR c <> 5 THEN INCR fails
    IF x <> 4 * cw OR y <> ch THEN INCR fails
    PRINT #1, "after cell fails="; fails

    DIM a(1 TO 3) AS WSTRING
    a(1) = "Alpha"
    a(2) = "Beta"
    a(3) = "Gamma"
    v = a()
    DIM b(1 TO 5) AS WSTRING
    b() = v
    PRINT #1, "variant ub="; UBOUND(b)
    PRINT #1, "b1 len="; LEN(b(1)); " asc="; ASC(b(1), 1)
    PRINT #1, "b2 len="; LEN(b(2)); " asc="; ASC(b(2), 1)
    PRINT #1, "b3 len="; LEN(b(3)); " asc="; ASC(b(3), 1)
    IF UBOUND(b) <> 3 THEN INCR fails
    IF b(1) = "Alpha" THEN
        PRINT #1, "b1 ok"
    ELSE
        PRINT #1, "b1 bad"
        INCR fails
    END IF
    IF b(2) = "Beta" THEN
        PRINT #1, "b2 ok"
    ELSE
        PRINT #1, "b2 bad"
        INCR fails
    END IF
    IF b(3) = "Gamma" THEN
        PRINT #1, "b3 ok"
    ELSE
        PRINT #1, "b3 bad"
        INCR fails
    END IF

    PRINT #1, "batch402 failures="; fails
    PRINT #1, "FAILURES:"; fails
    CLOSE #1
    FUNCTION = fails
END FUNCTION