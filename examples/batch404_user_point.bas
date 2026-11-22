#COMPILE EXE
#DIM ALL

' batch 404: a program that declares its own POINT type must keep it.  The
' built-in POINT/POINTAPI fallback is registered only when the source does not
' declare a type with that name, so this layout stays 12 bytes instead of
' being replaced by the compiler's 8-byte POINT.

TYPE POINT
    x AS LONG
    y AS LONG
    z AS LONG
END TYPE

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL p AS POINT

    fails = 0
    p.x = 10
    p.y = 20
    p.z = 30
    PRINT "sizeof_user_point="; SIZEOF(POINT)
    PRINT "values="; p.x; ","; p.y; ","; p.z
    IF SIZEOF(POINT) <> 12 THEN INCR fails
    IF p.x <> 10 OR p.y <> 20 OR p.z <> 30 THEN INCR fails
    PRINT "FAILURES:"; fails
    FUNCTION = fails
END FUNCTION
