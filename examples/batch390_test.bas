#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 390 test
' VERSION$() must report the version this compiler was built from.
DECLARE FUNCTION PBMAIN() AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL v AS STRING
    LOCAL bare AS STRING
    LOCAL fails AS LONG

    fails = 0
    v = VERSION$()
    bare = VERSION$
    PRINT "VERSION=" + v
    IF v <> bare THEN
        PRINT "bare form differs"
        fails = fails + 1
    END IF
    IF v <> "v0.2.178" THEN
        PRINT "unexpected VERSION$ value"
        fails = fails + 1
    END IF

    PRINT "FAILURES: " + STR$(fails)
    FUNCTION = fails
END FUNCTION
