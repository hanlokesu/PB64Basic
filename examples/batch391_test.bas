#COMPILE EXE
#DIM ALL

TYPE B391FIXED
    F AS STRING * 5
END TYPE

' batch 391: embedded-NUL (CHR$(0)) payloads must keep their full BSTR length
' through string alignment, OEM conversion, fixed/ASCIIZ conversion, and
' binary PUT$/GET$ round-trips.
FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL s AS STRING
    LOCAL a AS STRING * 5
    LOCAL b AS STRING * 5
    LOCAL c AS STRING * 5
    LOCAL d AS STRING * 5
    LOCAL g AS STRING
    LOCAL o AS STRING
    LOCAL p AS STRING
    LOCAL t AS STRING
    LOCAL rpos AS LONG
    LOCAL ver AS STRING
    LOCAL xf AS STRING * 5
    LOCAL xd AS STRING
    LOCAL xt AS B391FIXED
    LOCAL az AS ASCIIZ * 8
    LOCAL in5 AS STRING * 5
    LOCAL inz AS ASCIIZ * 5
    LOCAL li5 AS STRING * 5
    LOCAL liz AS ASCIIZ * 5
    LOCAL bv AS BYTE
    fails = 0
    ver = VERSION$()
    IF ver <> "v0.2.179" THEN INCR fails

    s = "A" + CHR$(0) + "B"
    IF LEN(s) <> 3 THEN INCR fails
    IF ASC(s, 1) <> 65 THEN INCR fails
    IF ASC(s, 2) <> 0 THEN INCR fails
    IF ASC(s, 3) <> 66 THEN INCR fails

    xf = SPACE$(5)
    LSET xf = s
    xd = xf
    IF LEN(xd) <> 5 THEN INCR fails
    IF ASC(xd, 1) <> 65 THEN INCR fails
    IF ASC(xd, 2) <> 0 THEN INCR fails
    IF ASC(xd, 3) <> 66 THEN INCR fails
    IF ASC(xd, 4) <> 32 THEN INCR fails
    IF ASC(xd, 5) <> 32 THEN INCR fails

    xt.F = s
    xd = xt.F
    IF LEN(xd) <> 5 THEN INCR fails
    IF ASC(xd, 1) <> 65 THEN INCR fails
    IF ASC(xd, 2) <> 0 THEN INCR fails
    IF ASC(xd, 3) <> 66 THEN INCR fails
    IF ASC(xd, 4) <> 32 THEN INCR fails
    IF ASC(xd, 5) <> 32 THEN INCR fails

    az = "AB"
    IF LEN(az) <> 2 THEN INCR fails
    IF ASC(az, 3) <> -1 THEN INCR fails
    az = s
    IF LEN(az) <> 1 THEN INCR fails
    IF ASC(az, 2) <> -1 THEN INCR fails
    xd = az
    IF LEN(xd) <> 1 THEN INCR fails
    IF ASC(xd, 1) <> 65 THEN INCR fails

    a = SPACE$(5)
    CSET a = s
    IF LEN(a) <> 5 THEN INCR fails
    IF ASC(a, 1) <> 32 THEN INCR fails
    IF ASC(a, 2) <> 65 THEN INCR fails
    IF ASC(a, 3) <> 0 THEN INCR fails
    IF ASC(a, 4) <> 66 THEN INCR fails
    IF ASC(a, 5) <> 32 THEN INCR fails

    b = SPACE$(5)
    LSET b = s
    IF LEN(b) <> 5 THEN INCR fails
    IF ASC(b, 1) <> 65 THEN INCR fails
    IF ASC(b, 2) <> 0 THEN INCR fails
    IF ASC(b, 3) <> 66 THEN INCR fails
    IF ASC(b, 4) <> 32 THEN INCR fails
    IF ASC(b, 5) <> 32 THEN INCR fails

    c = SPACE$(5)
    RSET c = s
    IF LEN(c) <> 5 THEN INCR fails
    IF ASC(c, 1) <> 32 THEN INCR fails
    IF ASC(c, 2) <> 32 THEN INCR fails
    IF ASC(c, 3) <> 65 THEN INCR fails
    IF ASC(c, 4) <> 0 THEN INCR fails
    IF ASC(c, 5) <> 66 THEN INCR fails

    d = s
    IF LEN(d) <> 5 THEN INCR fails
    IF ASC(d, 1) <> 65 THEN INCR fails
    IF ASC(d, 2) <> 0 THEN INCR fails
    IF ASC(d, 3) <> 66 THEN INCR fails
    IF ASC(d, 4) <> 32 THEN INCR fails
    IF ASC(d, 5) <> 32 THEN INCR fails

    o = CHRTOOEM$(s)
    p = OEMTOCHR$(o)
    IF LEN(p) <> 3 THEN INCR fails
    IF ASC(p, 1) <> 65 THEN INCR fails
    IF ASC(p, 2) <> 0 THEN INCR fails
    IF ASC(p, 3) <> 66 THEN INCR fails

    OPEN "batch391_in.txt" FOR OUTPUT AS #5
    PRINT #5, "AB"
    CLOSE #5
    OPEN "batch391_in.txt" FOR INPUT AS #6
    INPUT #6, in5
    CLOSE #6
    OPEN "batch391_in.txt" FOR INPUT AS #7
    INPUT #7, inz
    CLOSE #7
    IF LEN(in5) <> 5 THEN INCR fails
    IF ASC(in5, 1) <> 65 THEN INCR fails
    IF ASC(in5, 2) <> 66 THEN INCR fails
    IF ASC(in5, 3) <> 32 THEN INCR fails
    IF ASC(in5, 4) <> 32 THEN INCR fails
    IF ASC(in5, 5) <> 32 THEN INCR fails
    IF LEN(inz) <> 2 THEN INCR fails
    IF ASC(inz, 1) <> 65 THEN INCR fails
    IF ASC(inz, 2) <> 66 THEN INCR fails
    IF ASC(inz, 3) <> -1 THEN INCR fails

    OPEN "batch391_in.txt" FOR INPUT AS #8
    LINE INPUT #8, li5
    CLOSE #8
    OPEN "batch391_in.txt" FOR INPUT AS #10
    LINE INPUT #10, liz
    CLOSE #10
    IF LEN(li5) <> 5 THEN INCR fails
    IF ASC(li5, 3) <> 32 THEN INCR fails
    IF LEN(liz) <> 2 THEN INCR fails
    IF ASC(liz, 3) <> -1 THEN INCR fails

    OPEN "batch391_nul.bin" FOR BINARY AS #1
    PUT$ #1, s
    CLOSE #1
    OPEN "batch391_nul.bin" FOR BINARY AS #2
    GET$ #2, 3, g
    CLOSE #2
    IF LEN(g) <> 3 THEN INCR fails
    IF ASC(g, 1) <> 65 THEN INCR fails
    IF ASC(g, 2) <> 0 THEN INCR fails
    IF ASC(g, 3) <> 66 THEN INCR fails

    t = REMOVE$(s, CHR$(0))
    IF LEN(t) <> 2 THEN INCR fails
    IF ASC(t, 1) <> 65 THEN INCR fails
    IF ASC(t, 2) <> 66 THEN INCR fails

    REGREPL "zzz" IN s WITH "X" TO rpos, t
    IF rpos <> 0 THEN INCR fails
    IF LEN(t) <> 3 THEN INCR fails
    IF ASC(t, 1) <> 65 THEN INCR fails
    IF ASC(t, 2) <> 0 THEN INCR fails
    IF ASC(t, 3) <> 66 THEN INCR fails

    bv = 221
    ROTATE RIGHT bv, 1
    IF bv <> 238 THEN INCR fails
    bv = 221
    SHIFT SIGNED RIGHT bv, 1
    IF bv <> 238 THEN INCR fails

    PRINT "batch391 NUL-payload checks done"
    OPEN "batch391_nul_result.txt" FOR OUTPUT AS #9
    PRINT #9, "batch391 failures="; fails
    CLOSE #9
    FUNCTION = fails
END FUNCTION