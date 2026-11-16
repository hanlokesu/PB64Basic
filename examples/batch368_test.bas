#COMPILE EXE
#DIM ALL

' batch 368: TXT pseudo-object.  Exercises every documented member without
' blocking: TXT.WAITKEY$ and TXT.LINE.INPUT are compiled inside a branch that
' can never run on a machine with no pending key, so the automated gate does
' not hang on a headless stdin.

FUNCTION PBMAIN () AS LONG
    LOCAL hWin  AS LONG
    LOCAL h2    AS LONG
    LOCAL r     AS LONG
    LOCAL c     AS LONG
    LOCAL s     AS STRING
    LOCAL fails AS LONG

    ' Before TXT.WINDOW exists the documented behaviour is: INKEY$ is empty,
    ' INSTAT is FALSE, PRINT/CLS/CELL are harmless no-ops.
    s = TXT.INKEY$
    IF LEN(s) <> 0 THEN
        PRINT "INKEY$ without a window"
        INCR fails
    END IF
    IF TXT.INSTAT <> 0 THEN
        PRINT "INSTAT without a window"
        INCR fails
    END IF
    TXT.PRINT "ignored"
    TXT.CLS
    TXT.CELL TO r, c
    IF r <> 1 OR c <> 1 THEN
        PRINT "CELL without a window "; r; ","; c
        INCR fails
    END IF

    TXT.WINDOW("PB64 TXT test", 20, 20, 10, 40) TO hWin
    IF hWin = 0 THEN
        PRINT "TXT.WINDOW failed"
        INCR fails
        GOTO done
    END IF

    ' A second text window while one exists must fail, not hijack the first.
    TXT.WINDOW("second", 40, 40, 5, 20) TO h2
    IF h2 <> 0 THEN
        PRINT "second TXT.WINDOW unexpectedly succeeded"
        INCR fails
    END IF

    TXT.COLOR = RGB(255, 0, 0)
    TXT.PRINT "Hello"; SPC(2); "TXT"
    TXT.PRINT "zone1", "zone2"
    TXT.PRINT "tab"; TAB(12); "x"

    TXT.CELL = 4, 6
    TXT.CELL TO r, c
    IF r <> 4 OR c <> 6 THEN
        PRINT "TXT.CELL set/get "; r; ","; c
        INCR fails
    END IF
    TXT.PRINT "cell"
    ' A completed TXT.PRINT moves the cursor to column 1 of the next row.
    TXT.CELL TO r, c
    IF r <> 5 OR c <> 1 THEN
        PRINT "TXT.PRINT cursor "; r; ","; c
        INCR fails
    END IF

    TXT.CELL = 4, 6
    TXT.PRINT "cell";
    TXT.CELL TO r, c
    IF r <> 4 OR c <> 10 THEN
        PRINT "TXT.PRINT no-newline cursor "; r; ","; c
        INCR fails
    END IF

    TXT.COL TO c
    IF c <> 10 THEN
        PRINT "TXT.COL "; c
        INCR fails
    END IF
    TXT.ROW TO r
    IF r <> 4 THEN
        PRINT "TXT.ROW "; r
        INCR fails
    END IF
    TXT.COL = 3
    TXT.ROW = 5
    TXT.COL TO c
    TXT.ROW TO r
    IF c <> 3 OR r <> 5 THEN
        PRINT "TXT.COL/ROW set/get "; r; ","; c
        INCR fails
    END IF

    s = TXT.INKEY$
    IF LEN(s) <> 0 THEN
        PRINT "unexpected key: "; s
        INCR fails
    END IF
    IF TXT.INSTAT <> 0 THEN
        PRINT "unexpected INSTAT"
        INCR fails
        ' Compiled, never run in the automated gate: both reads block.
        TXT.WAITKEY$
        s = TXT.WAITKEY$
        TXT.LINE.INPUT("line: ", s)
    END IF

    TXT.CLS
    TXT.PRINT "after cls"

    TXT.END
    ' TXT.END with no window is a documented no-op.
    TXT.END

done:
    IF fails = 0 THEN
        PRINT "=== FAILURES:  0"
    ELSE
        PRINT "=== FAILURES: "; fails
    END IF
    FUNCTION = fails
END FUNCTION
