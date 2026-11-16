' batch363_test.bas - sequential CHR=WIDE UTF-16 layer (batch 363).
' Covers UTF-16LE write formatting, wide CRLF, LINE INPUT# / INPUT# lexing,
' fixed WSTRING capacity/padding, and the 0x1A end-of-file marker.
' PBWin10 probe10 produced the same values with fails=0.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL fails AS LONG
    LOCAL fw AS WSTRING * 8
    LOCAL s AS WSTRING
    LOCAL n AS LONG
    LOCAL d AS DOUBLE
    LOCAL r AS LONG
    LOCAL w AS LONG
    LOCAL e AS LONG
    LOCAL sc AS LONG
    OPEN "batch363_wide.out" FOR OUTPUT AS #1 CHR=WIDE
    PRINT #1, "ABC"
    PRINT #1, "DEF";
    PRINT #1, "GHI"
    PRINT #1, 123
    fw = "ABC"
    PRINT #1, fw
    PRINT #1, fw; "X"
    WRITE #1, fw, "Y"
    WRITE #1, "Q", 42, 1.25
    CLOSE #1
    OPEN "batch363_wide.out" FOR INPUT AS #1 CHR=WIDE
    LINE INPUT #1, s
    IF s <> "ABC" THEN fails = fails + 1
    SELECT CASE s
    CASE "ABC"
        sc = sc + 1
    CASE ELSE
        fails = fails + 1
    END SELECT
    LINE INPUT #1, s
    IF s <> "DEFGHI" THEN fails = fails + 1
    LINE INPUT #1, s
    IF s <> " 123 " THEN fails = fails + 1
    LINE INPUT #1, s
    IF s <> "ABC     " THEN fails = fails + 1
    LINE INPUT #1, s
    IF s <> "ABC     X" THEN fails = fails + 1
    LINE INPUT #1, s
    IF s <> CHR$(34) & "ABC     " & CHR$(34) & "," & CHR$(34) & "Y" & CHR$(34) THEN fails = fails + 1
    LINE INPUT #1, s
    IF s <> CHR$(34) & "Q" & CHR$(34) & ",42,1.25" THEN fails = fails + 1
    CLOSE #1
    OPEN "batch363_in.out" FOR OUTPUT AS #1 CHR=WIDE
    PRINT #1, " 42 , 7 "
    CLOSE #1
    OPEN "batch363_in.out" FOR INPUT AS #1 CHR=WIDE
    INPUT #1, n, d
    IF n <> 42 THEN fails = fails + 1
    IF d <> 7 THEN fails = fails + 1
    CLOSE #1
    OPEN "batch363_line.out" FOR OUTPUT AS #1 CHR=WIDE
    PRINT #1, "Z"
    CLOSE #1
    OPEN "batch363_line.out" FOR INPUT AS #1 CHR=WIDE
    LINE INPUT #1, fw
    IF LEN(fw) <> 8 THEN fails = fails + 1
    IF fw <> "Z       " THEN fails = fails + 1
    CLOSE #1
    OPEN "batch363_eof.out" FOR OUTPUT AS #1 CHR=WIDE
    PRINT #1, "A"; CHR$(26); "B"
    CLOSE #1
    OPEN "batch363_eof.out" FOR INPUT AS #1 CHR=WIDE
    FILESCAN #1, RECORDS TO r, WIDTH TO w
    IF r <> 1 THEN fails = fails + 1
    IF w <> 1 THEN fails = fails + 1
    LINE INPUT #1, s
    IF s <> "A" THEN fails = fails + 1
    e = EOF(#1)
    IF e <> -1 THEN fails = fails + 1
    IF sc <> 1 THEN fails = fails + 1
    CLOSE #1
    OPEN "batch363_out.txt" FOR OUTPUT AS #1
    PRINT #1, "fails="; fails
    CLOSE #1
    IF fails <> 0 THEN FUNCTION = 1 ELSE FUNCTION = 0
END FUNCTION