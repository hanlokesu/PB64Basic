' batch272_test.bas - ARRAY SORT / ARRAY SCAN custom COLLATE (256-char weight
' string, CHW ARRAY_SORT_statement.htm / ARRAY_SCAN_statement.htm) plus the
' CHR$(a TO b) range form it depends on.  Uses STRREVERSE$(CHR$(0 TO 255)) as
' the descending weight table (CHW example).  Output to b272_out.txt.
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    DIM a(1 TO 3) AS STRING
    LOCAL c AS STRING
    LOCAL r1 AS LONG
    LOCAL r2 AS LONG
    LOCAL s1 AS STRING
    LOCAL s2 AS STRING
    LOCAL s3 AS STRING
    LOCAL failures AS LONG
    failures = 0

    c = STRREVERSE$(CHR$(0 TO 255))

    a(1) = "apple"
    a(2) = "banana"
    a(3) = "cherry"
    ARRAY SCAN a(), COLLATE c, > "banana", TO r1
    ARRAY SORT a(), COLLATE c
    s1 = a(1)
    s2 = a(2)
    s3 = a(3)

    a(1) = "apple"
    a(2) = "banana"
    a(3) = "cherry"
    ARRAY SCAN a(), COLLATE UCASE, = "BANANA", TO r2

    IF r1 <> 1 THEN INCR failures
    IF s1 <> "cherry" THEN INCR failures
    IF s2 <> "banana" THEN INCR failures
    IF s3 <> "apple" THEN INCR failures
    IF r2 <> 2 THEN INCR failures

    OPEN "b272_out.txt" FOR OUTPUT AS #1
    IF failures = 0 THEN
        PRINT #1, "batch272 test ok"
    ELSE
        PRINT #1, "FAILURES: " + LTRIM$(STR$(failures))
    END IF
    CLOSE #1
    FUNCTION = failures
END FUNCTION
