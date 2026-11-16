#RESOURCE PBR, "batch367_test.pbr"

FUNCTION PBMAIN () AS LONG
    LOCAL s AS STRING
    LOCAL fails AS LONG

    s = RESOURCE$(STRING, 100)
    IF LEN(s) <> 12 THEN
        PRINT "RESOURCE$ length "; LEN(s)
        INCR fails
    ELSEIF s <> "PB64 PBR OK!" THEN
        PRINT "bad payload: ["; s; "]"
        INCR fails
    END IF

    IF fails = 0 THEN
        PRINT "=== FAILURES:  0"
    ELSE
        PRINT "=== FAILURES:  1"
    END IF
    FUNCTION = fails
END FUNCTION
