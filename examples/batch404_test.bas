#COMPILE EXE
#DIM ALL

' batch 404: the compiler must expose POINT/POINTAPI before any TYPE field is
' laid out.  José Roca's WinDef.inc deliberately leaves POINT undefined because
' the PB compiler supplies it; a late registration made every `AS POINT` field
' fail with Error 524.
'
' This source also includes a header whose #INCLUDE line has no closing quote,
' exactly like several lines shipped in the José Roca WINAPI set (for example
' ws2def.inc's `#INCLUDE ONCE "inaddr.inc`).  PBWin accepts the path at EOL.

#INCLUDE ONCE "batch404_include_quote.inc

TYPE B404_RECT2
    pt1 AS POINT
    pt2 AS POINTAPI
END TYPE

TYPE B404_MSG2
    hwnd AS DWORD
    pt AS POINT
END TYPE

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL p AS POINT
    LOCAL r AS B404_RECT2
    LOCAL m AS B404_MSG2
    LOCAL q AS B404_QUOTED_TYPE

    fails = 0

    PRINT "sizeof_point="; SIZEOF(POINT)
    PRINT "sizeof_pointapi="; SIZEOF(POINTAPI)
    PRINT "sizeof_rect2="; SIZEOF(B404_RECT2)
    PRINT "sizeof_msg2="; SIZEOF(B404_MSG2)

    IF SIZEOF(POINT) <> 8 THEN INCR fails
    IF SIZEOF(POINTAPI) <> 8 THEN INCR fails
    IF SIZEOF(B404_RECT2) <> 16 THEN INCR fails
    ' B404_MSG2 is DWORD + POINT; the struct's natural alignment may round it to
    ' 16 on x64.  Only require that the POINT member is fully contained.
    IF SIZEOF(B404_MSG2) < 12 THEN INCR fails

    p.x = 11
    p.y = -22
    r.pt1.x = 33
    r.pt1.y = 44
    r.pt2.x = -55
    r.pt2.y = 66
    m.pt.x = 77
    m.pt.y = -88

    PRINT "p="; p.x; ","; p.y
    PRINT "r1="; r.pt1.x; ","; r.pt1.y
    PRINT "r2="; r.pt2.x; ","; r.pt2.y
    PRINT "m="; m.pt.x; ","; m.pt.y

    IF p.x <> 11 OR p.y <> -22 THEN INCR fails
    IF r.pt1.x <> 33 OR r.pt1.y <> 44 THEN INCR fails
    IF r.pt2.x <> -55 OR r.pt2.y <> 66 THEN INCR fails
    IF m.pt.x <> 77 OR m.pt.y <> -88 THEN INCR fails

    q.Value = 1234
    q.Marker = 5678
    PRINT "quoted_include="; q.Value; ","; q.Marker
    IF q.Value <> 1234 OR q.Marker <> 5678 THEN INCR fails

    PRINT "FAILURES:"; fails
    FUNCTION = fails
END FUNCTION
