#COMPILE EXE
#DIM ALL

' batch 385: byte-string functions must use the BSTR byte length.
' PBWin10 reference (same source, file-output probe): LEN=3, LEFT=3,
' RIGHT=2, MID=2, INSTR=3, LTRIM=3, RTRIM=3 for "A" + CHR$(0) + "B".

GLOBAL g_fail AS LONG

TYPE T385
    f AS STRING * 6
END TYPE

SUB CheckLong(BYVAL got AS LONG, BYVAL want AS LONG, BYVAL what AS STRING)
    IF got <> want THEN
        PRINT "FAIL "; what; " got="; got; " want="; want
        g_fail = g_fail + 1
    END IF
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL s AS STRING
    LOCAL s2 AS STRING
    LOCAL z AS STRING
    LOCAL d AS STRING
    LOCAL p1 AS STRING
    LOCAL p2 AS STRING
    LOCAL f AS STRING * 6
    LOCAL t AS T385

    s = "A" + CHR$(0) + "B"
    CheckLong LEN(s), 3, "LEN embedded NUL"

    z = LEFT$(s, 3)
    CheckLong LEN(z), 3, "LEFT$ embedded NUL length"
    IF LEN(z) >= 3 THEN
        CheckLong ASC(z, 1), 65, "LEFT$ byte 1"
        CheckLong ASC(z, 2), 0, "LEFT$ embedded NUL byte"
        CheckLong ASC(z, 3), 66, "LEFT$ byte 3"
    END IF

    z = RIGHT$(s, 2)
    CheckLong LEN(z), 2, "RIGHT$ embedded NUL length"
    IF LEN(z) >= 2 THEN
        CheckLong ASC(z, 1), 0, "RIGHT$ embedded NUL byte"
        CheckLong ASC(z, 2), 66, "RIGHT$ byte 2"
    END IF

    z = MID$(s, 2, 2)
    CheckLong LEN(z), 2, "MID$ embedded NUL length"
    IF LEN(z) >= 2 THEN
        CheckLong ASC(z, 1), 0, "MID$ embedded NUL byte"
        CheckLong ASC(z, 2), 66, "MID$ byte 2"
    END IF

    CheckLong INSTR(s, "B"), 3, "INSTR after embedded NUL"
    CheckLong INSTR(2, s, "B"), 3, "INSTR start after embedded NUL"

    z = LTRIM$("  " + s)
    CheckLong LEN(z), 3, "LTRIM$ embedded NUL length"
    z = RTRIM$(s + "  ")
    CheckLong LEN(z), 3, "RTRIM$ embedded NUL length"

    CheckLong TALLY(s, "B"), 1, "TALLY after embedded NUL"
    CheckLong TALLY(s, CHR$(0)), 1, "TALLY embedded NUL byte"
    CheckLong VERIFY(s, "AB"), 2, "VERIFY embedded NUL pass"
    CheckLong VERIFY(s, "A"), 2, "VERIFY embedded NUL fail"

    z = REMOVE$(s, "B")
    CheckLong LEN(z), 2, "REMOVE$ trailing byte length"
    IF LEN(z) >= 2 THEN
        CheckLong ASC(z, 1), 65, "REMOVE$ byte 1"
        CheckLong ASC(z, 2), 0, "REMOVE$ embedded NUL byte"
    END IF
    z = REMOVE$(s, CHR$(0))
    CheckLong LEN(z), 2, "REMOVE$ embedded NUL length"
    IF LEN(z) >= 2 THEN
        CheckLong ASC(z, 1), 65, "REMOVE$ NUL byte 1"
        CheckLong ASC(z, 2), 66, "REMOVE$ NUL byte 2"
    END IF

    z = UCASE$(s)
    CheckLong LEN(z), 3, "UCASE$ embedded NUL length"
    IF LEN(z) >= 2 THEN CheckLong ASC(z, 2), 0, "UCASE$ keeps embedded NUL"
    z = LCASE$(s)
    CheckLong LEN(z), 3, "LCASE$ embedded NUL length"
    IF LEN(z) >= 2 THEN CheckLong ASC(z, 2), 0, "LCASE$ keeps embedded NUL"

    s2 = s
    REPLACE "B" WITH "C" IN s2
    CheckLong LEN(s2), 3, "REPLACE embedded NUL length"
    IF LEN(s2) >= 3 THEN
        CheckLong ASC(s2, 1), 65, "REPLACE byte 1"
        CheckLong ASC(s2, 2), 0, "REPLACE embedded NUL byte"
        CheckLong ASC(s2, 3), 67, "REPLACE byte 3"
    END IF
    s2 = s
    REPLACE ANY CHR$(0) WITH "Z" IN s2
    CheckLong LEN(s2), 3, "REPLACE ANY embedded NUL length"
    IF LEN(s2) >= 3 THEN
        CheckLong ASC(s2, 1), 65, "REPLACE ANY byte 1"
        CheckLong ASC(s2, 2), 90, "REPLACE ANY replaced NUL"
        CheckLong ASC(s2, 3), 66, "REPLACE ANY byte 3"
    END IF

    d = SPACE$(6)
    LSET d = s
    CheckLong LEN(d), 6, "LSET dynamic width"
    CheckLong ASC(d, 1), 65, "LSET dynamic byte 1"
    CheckLong ASC(d, 2), 0, "LSET dynamic embedded NUL"
    CheckLong ASC(d, 3), 66, "LSET dynamic byte 3"
    CheckLong ASC(d, 4), 32, "LSET dynamic pad"

    d = SPACE$(6)
    RSET d = s
    CheckLong LEN(d), 6, "RSET dynamic width"
    CheckLong ASC(d, 4), 65, "RSET dynamic byte 4"
    CheckLong ASC(d, 5), 0, "RSET dynamic embedded NUL"
    CheckLong ASC(d, 6), 66, "RSET dynamic byte 6"

    d = SPACE$(6)
    CSET d = s
    CheckLong LEN(d), 6, "CSET dynamic width"
    CheckLong ASC(d, 1), 32, "CSET dynamic left pad"
    CheckLong ASC(d, 2), 65, "CSET dynamic byte 2"
    CheckLong ASC(d, 3), 0, "CSET dynamic embedded NUL"
    CheckLong ASC(d, 4), 66, "CSET dynamic byte 4"
    CheckLong ASC(d, 6), 32, "CSET dynamic right pad"

    LSET f = s
    CheckLong LEN(f), 6, "LSET fixed width"
    CheckLong ASC(f, 1), 65, "LSET fixed byte 1"
    CheckLong ASC(f, 2), 0, "LSET fixed embedded NUL"
    CheckLong ASC(f, 3), 66, "LSET fixed byte 3"
    CheckLong ASC(f, 4), 32, "LSET fixed pad"

    t.f = s
    CheckLong LEN(t.f), 6, "TYPE fixed width"
    CheckLong ASC(t.f, 1), 65, "TYPE fixed byte 1"
    CheckLong ASC(t.f, 2), 0, "TYPE fixed embedded NUL"
    CheckLong ASC(t.f, 3), 66, "TYPE fixed byte 3"
    CheckLong ASC(t.f, 4), 32, "TYPE fixed pad"

    SPLIT s, 2 TO p1, p2
    CheckLong LEN(p1), 2, "SPLIT part1 embedded NUL length"
    IF LEN(p1) >= 2 THEN
        CheckLong ASC(p1, 1), 65, "SPLIT part1 byte 1"
        CheckLong ASC(p1, 2), 0, "SPLIT part1 embedded NUL"
    END IF
    CheckLong LEN(p2), 1, "SPLIT part2 length"
    IF LEN(p2) >= 1 THEN CheckLong ASC(p2, 1), 66, "SPLIT part2 byte 1"

    PRINT "=== FAILURES: "; g_fail
    FUNCTION = 0
END FUNCTION
