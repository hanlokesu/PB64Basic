#COMPILE EXE
#DIM ALL

' batch 386: dynamic LSET/RSET must honor ABS and USING.
' PBWin10 reference: lu=XY******** ru=********XY la=XYcdefghij
' ra=abcdefghXY le=(empty) re=(empty) lt=123.

GLOBAL g_fail AS LONG

FUNCTION PBMAIN () AS LONG
    LOCAL s AS STRING
    LOCAL p AS STRING
    DIM arr(1 TO 3) AS STRING
    p = "*"
    s = "abcdefghij"
    LSET s = "XY" USING p
    IF s <> "XY********" THEN
        PRINT "FAIL LSET USING got=["; s; "]"
        g_fail = g_fail + 1
    END IF
    s = "abcdefghij"
    RSET s = "XY" USING p
    IF s <> "********XY" THEN
        PRINT "FAIL RSET USING got=["; s; "]"
        g_fail = g_fail + 1
    END IF
    s = "abcdefghij"
    LSET ABS s = "XY"
    IF s <> "XYcdefghij" THEN
        PRINT "FAIL LSET ABS got=["; s; "]"
        g_fail = g_fail + 1
    END IF
    s = "abcdefghij"
    RSET ABS s = "XY"
    IF s <> "abcdefghXY" THEN
        PRINT "FAIL RSET ABS got=["; s; "]"
        g_fail = g_fail + 1
    END IF
    s = ""
    LSET s = "XY"
    IF s <> "" THEN
        PRINT "FAIL LSET empty got=["; s; "]"
        g_fail = g_fail + 1
    END IF
    s = ""
    RSET s = "XY"
    IF s <> "" THEN
        PRINT "FAIL RSET empty got=["; s; "]"
        g_fail = g_fail + 1
    END IF
    s = "abc"
    LSET s = "123456"
    IF s <> "123" THEN
        PRINT "FAIL LSET truncate got=["; s; "]"
        g_fail = g_fail + 1
    END IF
    ' String-array elements may hold codegen constants.  ARRAY DELETE and
    ' ARRAY ARRAYIX must not SysFreeString them (batch 386 heap-corruption fix).
    arr(1) = "aa"
    arr(2) = "bb"
    arr(3) = "cc"
    ARRAY DELETE arr(1)
    IF arr(1) <> "bb" THEN
        PRINT "FAIL ARRAY DELETE[1] got=["; arr(1); "]"
        g_fail = g_fail + 1
    END IF
    IF arr(2) <> "cc" THEN
        PRINT "FAIL ARRAY DELETE[2] got=["; arr(2); "]"
        g_fail = g_fail + 1
    END IF
    IF arr(3) <> "" THEN
        PRINT "FAIL ARRAY DELETE[3] got=["; arr(3); "]"
        g_fail = g_fail + 1
    END IF
    arr(1) = "aa"
    arr(2) = "bb"
    arr(3) = "cc"
    ARRAY ARRAYIX arr()
    IF arr(1) <> "1" THEN
        PRINT "FAIL ARRAYIX[1] got=["; arr(1); "]"
        g_fail = g_fail + 1
    END IF
    IF arr(2) <> "2" THEN
        PRINT "FAIL ARRAYIX[2] got=["; arr(2); "]"
        g_fail = g_fail + 1
    END IF
    IF arr(3) <> "3" THEN
        PRINT "FAIL ARRAYIX[3] got=["; arr(3); "]"
        g_fail = g_fail + 1
    END IF
    arr(1) = "aa"
    arr(2) = "bb"
    arr(3) = "cc"
    ARRAY INSERT arr(2), "zz"
    IF arr(1) <> "aa" THEN
        PRINT "FAIL ARRAY INSERT[1] got=["; arr(1); "]"
        g_fail = g_fail + 1
    END IF
    IF arr(2) <> "zz" THEN
        PRINT "FAIL ARRAY INSERT[2] got=["; arr(2); "]"
        g_fail = g_fail + 1
    END IF
    IF arr(3) <> "bb" THEN
        PRINT "FAIL ARRAY INSERT[3] got=["; arr(3); "]"
        g_fail = g_fail + 1
    END IF
    PRINT "FAILURES: "; g_fail
    FUNCTION = g_fail
END FUNCTION
