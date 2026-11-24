#COMPILE EXE
FUNCTION PBMAIN () AS LONG
    IF LEFT$(VERSION$, 1) <> "v" THEN
        PRINT "FAIL version="; VERSION$
        FUNCTION = 1
        EXIT FUNCTION
    END IF
    PRINT "batch406 version string ok: "; VERSION$
    FUNCTION = 0
END FUNCTION
