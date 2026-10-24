' PB64Basic Enhanced - batch 269 test: OPEN FOR RANDOM LEN= record positioning
'   + PUT/GET with dynamic STRING and numeric variables (PBWin-measured layout)
' Covers: record number = (pos-1)*rec_len, dynamic STRING written as a 2-byte
'   LE length prefix + bytes (not padded to rec_len), numeric raw bytes.
#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL t AS STRING
    LOCAL x AS LONG
    LOCAL y AS LONG
    LOCAL failures AS LONG
    failures = 0

    OPEN "b269_out.txt" FOR OUTPUT AS #2
    OPEN "b269.dat" FOR RANDOM AS #1 LEN = 20

    ' --- dynamic STRING record 1 -------------------------------------------
    s = "HELLO"
    PUT #1, 1, s
    GET #1, 1, t
    IF t <> "HELLO" OR LEN(t) <> 5 THEN
        PRINT #2, "FAIL rec1 string [" + t + "] len=" + LTRIM$(STR$(LEN(t)))
        INCR failures
    END IF

    ' --- dynamic STRING record 2 (positioning: offset 20) ------------------
    s = "XY"
    PUT #1, 2, s
    GET #1, 2, t
    IF t <> "XY" OR LEN(t) <> 2 THEN
        PRINT #2, "FAIL rec2 string [" + t + "] len=" + LTRIM$(STR$(LEN(t)))
        INCR failures
    END IF

    ' --- numeric record 2 (positioning shared with strings) ----------------
    x = 5
    PUT #1, 2, x
    GET #1, 2, y
    IF y <> 5 THEN
        PRINT #2, "FAIL rec2 numeric got=" + LTRIM$(STR$(y))
        INCR failures
    END IF

    CLOSE #1
    KILL "b269.dat"

    IF failures = 0 THEN
        PRINT #2, "batch269 test ok"
    ELSE
        PRINT #2, "FAILURES: " + LTRIM$(STR$(failures))
    END IF
    CLOSE #2
    FUNCTION = failures
END FUNCTION
