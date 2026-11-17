' batch379_wide_seek.bas - SEEK on a CHR=WIDE sequential file.
' CHW SEEK_statement: sequential SEEK is a BYTE position, not a character
' position, even when the sequential file is opened with CHR=WIDE.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL fails AS LONG
    LOCAL s AS WSTRING
    LOCAL p AS QUAD

    OPEN "batch379_wide_seek.dat" FOR OUTPUT AS #1 CHR=WIDE
    PRINT #1, "ABC"
    PRINT #1, "DEF"
    CLOSE #1

    OPEN "batch379_wide_seek.dat" FOR INPUT AS #1 CHR=WIDE
    SEEK #1, 11
    LINE INPUT #1, s
    p = SEEK(1)
    IF s <> "DEF" THEN INCR fails
    IF p <> 21 THEN INCR fails

    SEEK #1, 3
    LINE INPUT #1, s
    p = SEEK(1)
    IF s <> "BC" THEN INCR fails
    IF p <> 11 THEN INCR fails

    SEEK #1, 1
    LINE INPUT #1, s
    p = SEEK(1)
    IF s <> "ABC" THEN INCR fails
    IF p <> 11 THEN INCR fails
    CLOSE #1

    OPEN "batch379_wide_seek.txt" FOR OUTPUT AS #1
    PRINT #1, "batch379 wide seek, failures="; fails
    CLOSE #1
    IF fails <> 0 THEN FUNCTION = 1 ELSE FUNCTION = 0
END FUNCTION
