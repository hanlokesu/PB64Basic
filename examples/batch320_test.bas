#COMPILE EXE
#DIM ALL

' batch320 headless self-check: DECLARE calling-convention hardening must not
' break the legitimate internal FASTPROC block. Returns a nonzero exit code
' if any check fails.

FASTPROC FpAdd (BYVAL a AS LONG, BYVAL b AS LONG) AS LONG
END FASTPROC = a + b

FASTPROC FpZero () AS LONG
END FASTPROC = 0

FUNCTION PBMAIN() AS LONG
    LOCAL fails AS LONG
    IF FpAdd(20, 30) <> 50 THEN INCR fails
    IF FpAdd(-7, 7) <> 0 THEN INCR fails
    IF FpZero() <> 0 THEN INCR fails
    FUNCTION = fails
END FUNCTION
