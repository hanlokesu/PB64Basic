#COMPILE EXE
#CONSOLE ON
#DIM ALL

' batch 403: a missing dynamic LIB must yield E_FAIL, not a NULL call.

DECLARE FUNCTION Pb64MissingProc LIB "pb64_no_such.dll" ALIAS "NoSuchEntry" AS LONG

FUNCTION PBMAIN () AS LONG
    LOCAL rc AS LONG
    LOCAL fails AS LONG

    fails = 0
    OPEN "batch403_test_out.txt" FOR OUTPUT AS #1
    rc = Pb64MissingProc
    PRINT #1, "missing_rc="; rc
    PRINT #1, "missing_hex="; HEX$(rc)
    IF rc = 0 THEN INCR fails
    PRINT #1, "FAILURES:"; fails
    CLOSE #1
    FUNCTION = fails
END FUNCTION
