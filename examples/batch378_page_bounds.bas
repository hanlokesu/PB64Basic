#COMPILE EXE
#CONSOLE ON
#DIM ALL
GLOBAL g_fails AS LONG

FUNCTION CHECK(cond AS LONG, label AS STRING) AS LONG
    IF cond = 0 THEN
        CON.STDERR "FAIL: " + label + CHR$(13, 10)
        g_fails = g_fails + 1
    END IF
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL a AS LONG

    CON.PAGE.ACTIVE = 2
    CHECK ERR = 0, "ACTIVE valid clears ERR"
    CHECK CON.PAGE.ACTIVE = 2, "ACTIVE valid value"

    CON.PAGE.ACTIVE = 0
    CHECK ERR = 5, "ACTIVE 0 is Error 5"
    CHECK CON.PAGE.ACTIVE = 2, "ACTIVE 0 leaves value unchanged"

    CON.PAGE.ACTIVE = 9
    CHECK ERR = 5, "ACTIVE 9 is Error 5"
    CHECK CON.PAGE.ACTIVE = 2, "ACTIVE 9 leaves value unchanged"

    CON.PAGE.ACTIVE = 8
    CHECK ERR = 0, "ACTIVE 8 valid"
    CHECK CON.PAGE.ACTIVE = 8, "ACTIVE 8 value"

    CON.PAGE.VISIBLE = 3
    CHECK ERR = 0, "VISIBLE valid clears ERR"
    CHECK CON.PAGE.VISIBLE = 3, "VISIBLE valid value"

    CON.PAGE.VISIBLE = 0
    CHECK ERR = 5, "VISIBLE 0 is Error 5"
    CHECK CON.PAGE.VISIBLE = 3, "VISIBLE 0 leaves value unchanged"

    CON.PAGE.VISIBLE = 9
    CHECK ERR = 5, "VISIBLE 9 is Error 5"
    CHECK CON.PAGE.VISIBLE = 3, "VISIBLE 9 leaves value unchanged"

    CON.PAGE.VISIBLE = 8
    CHECK ERR = 0, "VISIBLE 8 valid"
    CHECK CON.PAGE.VISIBLE = 8, "VISIBLE 8 value"

    CON.PAGE.ACTIVE = 1
    CON.PAGE.VISIBLE = 1
    CHECK ERR = 0, "restore pages clears ERR"
    CHECK CON.PAGE.ACTIVE = 1, "restore ACTIVE"
    CHECK CON.PAGE.VISIBLE = 1, "restore VISIBLE"

    CON.STDOUT "batch378 page bounds, failures=" + STR$(g_fails) + CHR$(13, 10)
    FUNCTION = g_fails
END FUNCTION
