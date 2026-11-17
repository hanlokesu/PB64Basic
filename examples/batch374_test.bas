#COMPILE EXE
#CONSOLE ON
#DIM ALL

' batch 374: PB/CC console screen family.
' LOCATE, CURSORX/CURSORY, CON.CELL and the color/spacing helpers are
' standard PB/CC.  PRINT AT is a fork extension; official PB spells the
' same operation LOCATE row, col : PRINT text.

GLOBAL g_fails AS LONG

SUB Check(BYVAL cond AS LONG, BYVAL label AS STRING)
    IF cond = 0 THEN
        PRINT "FAIL "; label
        PRINT #1, "FAIL "; label
        INCR g_fails
    ELSE
        PRINT "PASS "; label
        PRINT #1, "PASS "; label
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL r AS LONG
    LOCAL c AS LONG
    LOCAL x AS LONG
    LOCAL y AS LONG

    OPEN "batch374_console_result.txt" FOR OUTPUT AS #1

    ' CLS must clear the screen and home the cursor to 1,1.
    ' Capture both coordinates before Check prints anything: Check's own
    ' PRINT moves the cursor and would invalidate the second read.
    CLS
    x = CURSORX
    y = CURSORY
    Check x = 1, "CLS col"
    Check y = 1, "CLS row"

    ' LOCATE is 1-based.  "LOCATE-OK" is 9 characters, so the cursor
    ' must finish at column 10.
    LOCATE 2, 1
    PRINT "LOCATE-OK";
    x = CURSORX
    y = CURSORY
    Check x = 10, "LOCATE/CURSORX"
    Check y = 2, "LOCATE/CURSORY"

    ' PRINT AT row, col: fork extension.  It positions before writing,
    ' then the absence of a trailing separator emits a newline.
    PRINT AT 3, 1, "PRINT-AT-OK"
    Check CON.CELL.ROW = 4, "PRINT AT row"
    Check CON.CELL.COL = 1, "PRINT AT col"

    ' Parenthesised position form.
    PRINT AT (4, 1), "PAREN-OK"

    ' CON.CELL setter: cursor to row 5, column 1; "CELL-SET-OK" is 11
    ' characters, so the cursor finishes at column 12.
    CON.CELL = 5, 1
    PRINT "CELL-SET-OK";
    x = CURSORX
    y = CURSORY
    Check y = 5, "CON.CELL set row"
    Check x = 12, "CON.CELL set col"

    ' CON.CELL TO returns both coordinates.  Put the cursor back at the
    ' position the previous capture observed; the Check calls above have
    ' since moved it.
    CON.CELL = 5, 13
    CON.CELL TO r, c
    Check r = 5, "CON.CELL TO row"
    Check c = 13, "CON.CELL TO col"

    ' PB/CC COLOR with the optional Count argument.  There is no direct
    ' attribute read-back, so this test verifies it does not disturb the
    ' cursor or the characters underneath.
    CON.CELL = 6, 13
    COLOR 0, 7, 3
    COLOR 7, 0
    x = CURSORX
    Check x = 13, "COLOR count keeps cursor"

    ' TAB(n) moves to 1-based column n; SPC(n) emits n spaces.
    LOCATE 7, 1
    PRINT TAB(10); "TAB-OK";
    x = CURSORX
    Check x = 16, "TAB/X"
    LOCATE 8, 1
    PRINT "A"; SPC(3); "B";
    x = CURSORX
    Check x = 6, "SPC/X"

    ' A position-only PRINT AT with a trailing semicolon must not emit a
    ' newline; it leaves the cursor exactly at the requested cell.
    PRINT AT 10, 1;
    x = CURSORX
    y = CURSORY
    Check x = 1, "PRINT AT no args col"
    Check y = 10, "PRINT AT no args row"

    IF g_fails = 0 THEN
        PRINT "ALL CONSOLE TESTS PASSED"
        PRINT #1, "ALL CONSOLE TESTS PASSED"
    ELSE
        PRINT "FAILURES: "; g_fails
        PRINT #1, "FAILURES: "; g_fails
    END IF
    CLOSE #1
    FUNCTION = g_fails
END FUNCTION
