' batch 405 release witness: GRAPHIC BOX negative-corner semantics and
' GRAPHIC COPY over the corrected i32 ABI. Headless: no windows or DOEVENTS.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION PBMAIN () AS LONG
    LOCAL h1 AS DWORD
    LOCAL h2 AS DWORD
    LOCAL px AS LONG
    LOCAL failures AS LONG
    failures = 0

    GRAPHIC BITMAP NEW 40, 40 TO h1
    GRAPHIC ATTACH h1, 0
    GRAPHIC CLEAR RGB(255, 255, 255)
    GRAPHIC BOX (0, 0)-(39, 39), -10, RGB(0, 0, 0), RGB(0, 0, 255)
    GRAPHIC GET PIXEL (2, 2) TO px
    IF px <> RGB(255, 255, 255) THEN
        INCR failures
        PRINT "FAIL negative corner pixel="; px
    END IF

    GRAPHIC CLEAR RGB(255, 255, 255)
    GRAPHIC BOX (39, 39)-(0, 0), -200, RGB(0, 0, 0), RGB(0, 255, 0)
    GRAPHIC GET PIXEL (2, 2) TO px
    IF px <> RGB(255, 255, 255) THEN
        INCR failures
        PRINT "FAIL reverse-coordinate negative corner pixel="; px
    END IF

    GRAPHIC BOX (0, 0)-(39, 39), 0, RGB(0, 0, 0), RGB(255, 0, 0)
    GRAPHIC GET PIXEL (2, 2) TO px
    IF px <> RGB(255, 0, 0) THEN
        INCR failures
        PRINT "FAIL sharp corner pixel="; px
    END IF

    GRAPHIC BITMAP NEW 20, 20 TO h2
    GRAPHIC ATTACH h2, 0
    GRAPHIC CLEAR RGB(0, 0, 0)
    GRAPHIC ATTACH h1, 0
    GRAPHIC COPY h2, 0, (0, 0)-(19, 19) TO (20, 0)
    GRAPHIC GET PIXEL (25, 3) TO px
    IF px <> RGB(0, 0, 0) THEN
        INCR failures
        PRINT "FAIL bitmap copy pixel="; px
    END IF

    GRAPHIC BITMAP END h2
    GRAPHIC BITMAP END h1
    PRINT "FAILURES:"; failures
    FUNCTION = failures
END FUNCTION
