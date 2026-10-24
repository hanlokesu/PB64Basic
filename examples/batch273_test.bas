' batch273_test.bas - DATE$/TIME$/DAYNAME$/MONTHNAME$/DEC$ dual-verified.
' CHW: DATE_system_variable.htm / TIME_system_variable.htm / DAYNAME$_function.htm
' / MONTHNAME$_function.htm / DEC$_function.htm.  Output to b273_out.txt.
' DEC$ uses banker's rounding (PBWin measured: DEC$(1.5)=2, DEC$(2.5)=2).
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL failures AS LONG
    LOCAL d AS STRING
    LOCAL t AS STRING
    LOCAL i AS LONG
    failures = 0
    OPEN "b273_out.txt" FOR OUTPUT AS #1
    d = DATE$
    t = TIME$
    IF LEN(d) <> 10 THEN INCR failures
    IF LEN(t) <> 8 THEN INCR failures
    FOR i = 0 TO 6
        SELECT CASE i
            CASE 0: IF DAYNAME$(i) <> "Sunday" THEN INCR failures
            CASE 1: IF DAYNAME$(i) <> "Monday" THEN INCR failures
            CASE 2: IF DAYNAME$(i) <> "Tuesday" THEN INCR failures
            CASE 3: IF DAYNAME$(i) <> "Wednesday" THEN INCR failures
            CASE 4: IF DAYNAME$(i) <> "Thursday" THEN INCR failures
            CASE 5: IF DAYNAME$(i) <> "Friday" THEN INCR failures
            CASE 6: IF DAYNAME$(i) <> "Saturday" THEN INCR failures
        END SELECT
    NEXT i
    FOR i = 1 TO 12
        SELECT CASE i
            CASE 1: IF MONTHNAME$(i) <> "January" THEN INCR failures
            CASE 2: IF MONTHNAME$(i) <> "February" THEN INCR failures
            CASE 3: IF MONTHNAME$(i) <> "March" THEN INCR failures
            CASE 4: IF MONTHNAME$(i) <> "April" THEN INCR failures
            CASE 5: IF MONTHNAME$(i) <> "May" THEN INCR failures
            CASE 6: IF MONTHNAME$(i) <> "June" THEN INCR failures
            CASE 7: IF MONTHNAME$(i) <> "July" THEN INCR failures
            CASE 8: IF MONTHNAME$(i) <> "August" THEN INCR failures
            CASE 9: IF MONTHNAME$(i) <> "September" THEN INCR failures
            CASE 10: IF MONTHNAME$(i) <> "October" THEN INCR failures
            CASE 11: IF MONTHNAME$(i) <> "November" THEN INCR failures
            CASE 12: IF MONTHNAME$(i) <> "December" THEN INCR failures
        END SELECT
    NEXT i
    IF DEC$(12345) <> "12345" THEN INCR failures
    IF DEC$(-123) <> "-123" THEN INCR failures
    IF DEC$(12345, 8) <> "00012345" THEN INCR failures
    IF DEC$(12345, -8) <> "   12345" THEN INCR failures
    IF DEC$(12345, 0, 3, 2) <> "   12345  " THEN INCR failures
    IF DEC$(1.6) <> "2" THEN INCR failures
    IF DEC$(-1.6) <> "-2" THEN INCR failures
    IF DEC$(1.5) <> "2" THEN INCR failures
    IF DEC$(2.5) <> "2" THEN INCR failures
    IF DEC$(3.5) <> "4" THEN INCR failures
    IF DEC$(-2.5) <> "-2" THEN INCR failures
    IF DEC$(0, 4) <> "0000" THEN INCR failures
    IF failures = 0 THEN
        PRINT #1, "batch273 test ok"
    ELSE
        PRINT #1, "FAILURES: " + LTRIM$(STR$(failures))
    END IF
    CLOSE #1
    FUNCTION = failures
END FUNCTION
