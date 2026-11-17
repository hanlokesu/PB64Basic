#COMPILE EXE
#DIM ALL

' batch 375: explicit calling conventions on FUNCTION/SUB.
' i686 must emit stdcall-decorated symbols for STDCALL/SDECL and a plain
' cdecl symbol for CDECL.  x64 has one platform convention.

GLOBAL g_total AS LONG
GLOBAL g_fails AS LONG

SUB Check(BYVAL cond AS LONG, BYVAL label AS STRING)
    IF cond = 0 THEN
        PRINT "FAIL "; label
        INCR g_fails
    ELSE
        PRINT "PASS "; label
    END IF
END SUB

FUNCTION AddStdcall STDCALL(BYVAL a AS LONG, BYVAL b AS LONG) AS LONG
    FUNCTION = a + b
END FUNCTION

FUNCTION AddSdecl SDECL(BYVAL a AS LONG, BYVAL b AS LONG) AS LONG
    FUNCTION = a + b
END FUNCTION

FUNCTION AddCdecl CDECL(BYVAL a AS LONG, BYVAL b AS LONG) AS LONG
    FUNCTION = a + b
END FUNCTION

SUB AccumStdcall STDCALL(BYVAL n AS LONG)
    g_total = g_total + n
END SUB

SUB AccumSdecl SDECL(BYVAL n AS LONG)
    g_total = g_total + n
END SUB

SUB AccumCdecl CDECL(BYVAL n AS LONG)
    g_total = g_total + n
END SUB

FUNCTION PBMAIN() AS LONG
    Check AddStdcall(20, 30) = 50, "STDCALL function"
    Check AddSdecl(20, 30) = 50, "SDECL function"
    Check AddCdecl(7, 8) = 15, "CDECL function"

    g_total = 0
    AccumStdcall 5
    AccumSdecl 6
    AccumCdecl 7
    Check g_total = 18, "mixed SUB calls"

    IF g_fails = 0 THEN
        PRINT "ALL CALLING CONVENTION TESTS PASSED"
    ELSE
        PRINT "FAILURES: "; g_fails
    END IF
    FUNCTION = g_fails
END FUNCTION
