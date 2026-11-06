#COMPILE EXE
#DIM ALL
' batch319_test.bas - DECLARE calling-convention hardening.
' Headless self-check: the legitimate FASTPROC procedure block (max two BYVAL
' LONG parameters, register-based, no stack frame) still works, and ordinary
' arithmetic is unaffected. No MSGBOX, WAITKEY$, DIALOG or INPUT. PBMAIN
' returns the failure count (nonzero exit) so a regression fails the run gate.
FASTPROC FpAdd (BYVAL a AS LONG, BYVAL b AS LONG) AS LONG
END FASTPROC = a + b

FUNCTION PBMAIN() AS LONG
    LOCAL fails AS LONG
    IF FpAdd(20, 30) <> 50 THEN INCR fails
    IF FpAdd(-5, 5) <> 0 THEN INCR fails
    IF (20 + 30) <> 50 THEN INCR fails
    FUNCTION = fails
END FUNCTION
