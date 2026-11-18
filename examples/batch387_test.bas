#COMPILE EXE
#DIM ALL

' batch 387: target-aware pointer width in TYPE layout and dynamic arrays.
' PBWin10 32-bit oracle: TYPE with DWORD PTR + LONG has SIZEOF 8; x64 has 12.
' A previous i686 build reported 12 because IrType::Ptr was hard-coded to 8.

GLOBAL g_fail AS LONG

TYPE PTRPAIR
    p AS DWORD PTR
    n AS LONG
END TYPE

FUNCTION PBMAIN () AS LONG
    LOCAL v AS PTRPAIR
    LOCAL ps AS LONG
    LOCAL sz AS LONG
    DIM da() AS STRING
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
    REDIM da(1 TO 3)
    da(1) = "aa"
    da(2) = "bb"
    da(3) = "cc"
    REDIM PRESERVE da(1 TO 5)
    da(4) = "dd"
    da(5) = "ee"
    IF da(1) <> "aa" OR da(2) <> "bb" OR da(3) <> "cc" _
       OR da(4) <> "dd" OR da(5) <> "ee" THEN
        PRINT "FAIL dynamic REDIM PRESERVE"
        g_fail = g_fail + 1
    END IF
    ARRAY DELETE da(2)
    IF da(1) <> "aa" OR da(2) <> "cc" OR da(3) <> "dd" _
       OR da(4) <> "ee" OR da(5) <> "" THEN
        PRINT "FAIL dynamic ARRAY DELETE"
        g_fail = g_fail + 1
    END IF
    PRINT "FAILURES: "; g_fail
    FUNCTION = g_fail
END FUNCTION
