#COMPILE EXE
#DIM ALL

' batch 389: a TYPE field declared AS INTEGER keeps its signed 16-bit value.
' Before this fix the i16 field load was zero-extended, so -32000 read back
' as 33536 (0x8300) when used in a 32-bit expression.

GLOBAL g_fail AS LONG

TYPE PAIR
    a AS INTEGER
    b AS INTEGER
END TYPE

FUNCTION PBMAIN () AS LONG
    LOCAL p AS PAIR
    LOCAL i AS INTEGER
    p.a = -32000
    p.b = 32767
    i = -12345
    IF p.a <> -32000 THEN
        PRINT "FAIL TYPE field negative got="; p.a
        g_fail = g_fail + 1
    END IF
    IF p.b <> 32767 THEN
        PRINT "FAIL TYPE field maximum got="; p.b
        g_fail = g_fail + 1
    END IF
    IF i <> -12345 THEN
        PRINT "FAIL local INTEGER negative got="; i
        g_fail = g_fail + 1
    END IF
    PRINT "FAILURES: "; g_fail
    FUNCTION = g_fail
END FUNCTION
