#COMPILE EXE
#CONSOLE ON
FUNCTION PBMAIN () AS LONG
    LOCAL n AS LONG
    LOCAL fails AS LONG

    fails = 0
    n = ABS(-42)
    IF n <> 42 THEN
        PRINT "batch380 FAIL: ABS"
        fails = fails + 1
    END IF
    IF fails = 0 THEN
        PRINT "batch380 PASS: help integration package"
    END IF
    FUNCTION = fails
END FUNCTION
