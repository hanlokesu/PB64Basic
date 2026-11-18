' batch382_open_var - OPEN ... AS #f with a valid FREEFILE value.
#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL f AS LONG
    LOCAL s AS STRING
    LOCAL fails AS LONG

    f = FREEFILE
    IF f < 1 OR f > 32767 THEN INCR fails
    OPEN "batch382_open_var.txt" FOR OUTPUT AS #f
    PRINT #f, "alpha"
    CLOSE #f

    f = FREEFILE
    OPEN "batch382_open_var.txt" FOR INPUT AS #f
    LINE INPUT #f, s
    CLOSE #f
    IF s <> "alpha" THEN INCR fails

    PRINT "batch382 open var, failures="; fails
    OPEN "batch382_open_var_result.txt" FOR OUTPUT AS #1
    PRINT #1, "batch382 open var, failures="; fails
    CLOSE #1
END FUNCTION