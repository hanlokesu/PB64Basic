#COMPILE EXE
#DIM ALL

' batch 388: JK-IDE dual-target add-on release smoke test.
' The Compilers menu itself is verified live through command IDs 55012/55028.
' This example verifies that the compiler selected by either entry can build
' a target-aware program: pointer width is 8 on x86_64 and 4 on i686.

GLOBAL g_fail AS LONG

TYPE PTRPAIR
    p AS DWORD PTR
    n AS LONG
END TYPE

FUNCTION PBMAIN () AS LONG
    LOCAL v AS PTRPAIR
    LOCAL ps AS LONG
    LOCAL sz AS LONG
    ps = SIZEOF(v.p)
    sz = SIZEOF(v)
    IF ps <> 4 AND ps <> 8 THEN
        PRINT "FAIL pointer width got="; ps
        g_fail = g_fail + 1
    END IF
    IF sz <> ps + 4 THEN
        PRINT "FAIL TYPE size got="; sz; " ptr="; ps
        g_fail = g_fail + 1
    END IF
    PRINT "batch388 target pointer width="; ps
    PRINT "FAILURES: "; g_fail
    FUNCTION = g_fail
END FUNCTION
