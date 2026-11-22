' batch 405: compile-only witness for runtime declarations that are easy to
' miss: GRAPHIC POLYGON/POLYLINE on a UDT and XPRINT LINE STEP (relative).
' The body is never called, so no printer or window is opened at run time.
#COMPILE EXE
#DIM ALL

TYPE P405
    count AS LONG
    x1 AS SINGLE
    y1 AS SINGLE
    x2 AS SINGLE
    y2 AS SINGLE
    x3 AS SINGLE
    y3 AS SINGLE
END TYPE

SUB DeclOnly()
    LOCAL p AS P405
    LOCAL h AS DWORD
    p.count = 3
    p.x1 = 10 : p.y1 = 10
    p.x2 = 80 : p.y2 = 10
    p.x3 = 45 : p.y3 = 80
    GRAPHIC BITMAP NEW 100, 100 TO h
    GRAPHIC ATTACH h, 0
    GRAPHIC POLYGON p
    GRAPHIC POLYLINE p
    XPRINT LINE STEP (10, 10)
    GRAPHIC BITMAP END h
END SUB

FUNCTION PBMAIN () AS LONG
    FUNCTION = 0
END FUNCTION
