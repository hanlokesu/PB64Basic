#COMPILE EXE
#CONSOLE ON
FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG

    fails = 0
    PRINT "batch381 help package smoke"

    IF ISFILE("jkide\PB64Basic.chm") OR ISFILE("..\jkide\PB64Basic.chm") THEN
        PRINT "batch381 OK: PB64Basic.chm"
    ELSE
        PRINT "batch381 FAIL: PB64Basic.chm missing"
        fails = fails + 1
    END IF

    IF ISFILE("jkide\PB64Basic.chw") OR ISFILE("..\jkide\PB64Basic.chw") THEN
        PRINT "batch381 OK: PB64Basic.chw"
    ELSE
        PRINT "batch381 FAIL: PB64Basic.chw missing"
        fails = fails + 1
    END IF

    IF fails = 0 THEN
        PRINT "batch381 PASS: help package files present"
    END IF
    FUNCTION = fails
END FUNCTION
