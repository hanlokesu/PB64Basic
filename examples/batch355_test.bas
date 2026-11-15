#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 355 test
' VERSION$() and the bare VERSION$ spelling return the release version this
' compiler was built from.  The value is baked in from the repository VERSION
' file at build time, so it is the same string the compiler banner and
' `PB64Basic --version` print.
DECLARE FUNCTION PBMAIN() AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL v AS STRING
    LOCAL bare AS STRING
    LOCAL i AS LONG
    LOCAL fails AS LONG

    fails = 0

    v = VERSION$()
    bare = VERSION$
    PRINT "VERSION=" + v
    PRINT "VERSION-bare=" + bare

    IF v <> bare THEN
        PRINT "bare form differs from VERSION$()"
        fails = fails + 1
    END IF

    IF LEN(v) >= 6 AND LEFT$(v, 2) = "v0" THEN
        PRINT "VERSION$ format ok"
    ELSE
        PRINT "VERSION$ format BAD"
        fails = fails + 1
    END IF

    ' BSTR ownership: every call allocates a fresh string that the assignment
    ' then releases.  A crash or short value here means the returned buffer is
    ' not really owned by the PB string machinery.
    FOR i = 1 TO 200
        v = VERSION$()
        IF LEN(v) < 6 THEN
            fails = fails + 1
        END IF
    NEXT i

    PRINT "FAILURES: " + STR$(fails)
    FUNCTION = 0
END FUNCTION
