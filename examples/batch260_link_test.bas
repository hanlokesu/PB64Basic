' batch260_link_test.bas - #LINK "sum.obj" external object module demo
' (batch260; see batch260_make_obj.txt for how sum.obj is built)
' Prints 50 (20 + 30) via the external sum() from sum.obj.
#COMPILE EXE
#LINK "sum.obj"
DECLARE FUNCTION sum ALIAS "sum" (BYVAL a AS LONG, BYVAL b AS LONG) AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL t AS LONG
    t = sum(20, 30)
    PRINT t
    FUNCTION = 0
END FUNCTION
