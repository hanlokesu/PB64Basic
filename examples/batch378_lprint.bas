#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN () AS LONG
    LOCAL s AS STRING
    LOCAL fails AS LONG
    fails = 0

    ' Case 1: semicolon between items is still a same-line continuation.
    OPEN "batch378_lprint1.txt" FOR OUTPUT AS #1
    CLOSE #1
    LPRINT ATTACH "batch378_lprint1.txt"
    LPRINT "A";
    LPRINT "B"
    LPRINT CLOSE
    OPEN "batch378_lprint1.txt" FOR INPUT AS #2
    LINE INPUT #2, s
    IF LEN(s) <> 2 OR s <> "AB" THEN INCR fails
    IF NOT EOF(2) THEN INCR fails
    CLOSE #2

    ' Case 2: a comma advances to the next 14-column print zone.
    OPEN "batch378_lprint2.txt" FOR OUTPUT AS #1
    CLOSE #1
    LPRINT ATTACH "batch378_lprint2.txt"
    LPRINT "A", "B"
    LPRINT CLOSE
    OPEN "batch378_lprint2.txt" FOR INPUT AS #2
    LINE INPUT #2, s
    IF LEN(s) <> 15 THEN INCR fails
    IF INSTR(s, "B") <> 15 THEN INCR fails
    IF NOT EOF(2) THEN INCR fails
    CLOSE #2

    ' Case 3: a trailing comma suppresses CRLF and advances the zone.
    OPEN "batch378_lprint3.txt" FOR OUTPUT AS #1
    CLOSE #1
    LPRINT ATTACH "batch378_lprint3.txt"
    LPRINT "A",
    LPRINT "B"
    LPRINT CLOSE
    OPEN "batch378_lprint3.txt" FOR INPUT AS #2
    LINE INPUT #2, s
    IF LEN(s) <> 15 THEN INCR fails
    IF INSTR(s, "B") <> 15 THEN INCR fails
    IF NOT EOF(2) THEN INCR fails
    CLOSE #2

    ' Case 4: a trailing semicolon suppresses CRLF with no zone padding.
    OPEN "batch378_lprint4.txt" FOR OUTPUT AS #1
    CLOSE #1
    LPRINT ATTACH "batch378_lprint4.txt"
    LPRINT "A";
    LPRINT CLOSE
    OPEN "batch378_lprint4.txt" FOR INPUT AS #2
    LINE INPUT #2, s
    IF LEN(s) <> 1 OR s <> "A" THEN INCR fails
    IF NOT EOF(2) THEN INCR fails
    CLOSE #2

    PRINT "batch378 lprint, failures="; fails
    OPEN "batch378_lprint_result.txt" FOR OUTPUT AS #3
    PRINT #3, "batch378 lprint, failures="; fails
    CLOSE #3
END FUNCTION
