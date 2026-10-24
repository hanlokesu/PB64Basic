' PB64Basic Enhanced - batch 270 test: PRINT # semicolon (no newline) + CRLF output
' Covers: trailing ; suppresses the CR/LF, colon statement separators, multi-item
'         PRINT # joins without spaces, and PRINT # files use CRLF (PBWin-measured).
#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL failures AS LONG
    failures = 0

    OPEN "b270_out.txt" FOR OUTPUT AS #2
    OPEN "b270.dat" FOR OUTPUT AS #3
    PRINT #3, "13456"; : PRINT #3, "789"
    PRINT #3, ""
    PRINT #3, "A"; "B"; "C"
    PRINT #3, ""
    CLOSE #3

    OPEN "b270.dat" FOR INPUT AS #1
    LINE INPUT #1, s
    IF s <> "13456789" THEN
        PRINT #2, "FAIL line1 [" + s + "]"
        INCR failures
    END IF
    LINE INPUT #1, s
    IF s <> "" THEN
        PRINT #2, "FAIL line2 [" + s + "]"
        INCR failures
    END IF
    LINE INPUT #1, s
    IF s <> "ABC" THEN
        PRINT #2, "FAIL line3 [" + s + "]"
        INCR failures
    END IF
    CLOSE #1
    KILL "b270.dat"

    IF failures = 0 THEN
        PRINT #2, "batch270 test ok"
    ELSE
        PRINT #2, "FAILURES: " + LTRIM$(STR$(failures))
    END IF
    CLOSE #2

    FUNCTION = failures
END FUNCTION
