' batch318_test.bas - R/S family (batch 318) double-run probe.
' Headless: writes results to batch318_out.txt via PRINT #1. No MSGBOX,
' no WAITKEY$, no DIALOG, no INPUT. Semicolon item separators are used so
' the output is byte-for-byte comparable with PBWin10.
#COMPILE EXE
#DIM ALL

GLOBAL c_c1 AS LONG
GLOBAL c_v1 AS LONG

FUNCTION cond1 AS LONG
    c_c1 = c_c1 + 1
    FUNCTION = 0
END FUNCTION

FUNCTION val1 AS LONG
    c_v1 = c_v1 + 1
    FUNCTION = 100
END FUNCTION

FUNCTION PBMAIN()
    LOCAL i AS LONG
    LOCAL ok AS LONG
    DIM ra(10) AS SINGLE
    LOCAL rz_i AS LONG
    LOCAL rz_s AS STRING
    DIM rz_arr(1 TO 3) AS LONG
    LOCAL rep AS STRING
    LOCAL remv AS STRING
    LOCAL rxtxt AS STRING
    LOCAL rxpat AS STRING
    LOCAL rxcap AS STRING
    LOCAL rxpos AS LONG
    LOCAL rxres AS STRING
    LOCAL bv AS BYTE
    LOCAL lv AS LONG
    LOCAL s AS STRING
    LOCAL sp1 AS STRING
    LOCAL sp2 AS STRING
    LOCAL sw1 AS LONG
    LOCAL sw2 AS LONG
    LOCAL sws1 AS STRING
    LOCAL sws2 AS STRING
    LOCAL reslen AS LONG

    OPEN "batch318_out.txt" FOR OUTPUT AS #1

    ' --- RANDOMIZE / RND same-seed reproducibility ---
    RANDOMIZE 42
    FOR i = 1 TO 10
        ra(i) = RND
    NEXT i
    RANDOMIZE 42
    ok = 1
    FOR i = 1 TO 10
        IF RND <> ra(i) THEN ok = 0
    NEXT i
    PRINT #1, "SAMESEED"; ok

    ' --- RESET zeroes variables and arrays ---
    rz_i = 42
    rz_s = "hello"
    rz_arr(1) = 11
    rz_arr(2) = 22
    rz_arr(3) = 33
    RESET rz_i
    RESET rz_s
    RESET rz_arr()
    PRINT #1, "RESET"; rz_i; rz_arr(1); rz_arr(2); rz_arr(3)
    PRINT #1, "RESETS"; "["; rz_s; "]"

    ' --- REPLACE ANY ---
    rep = "abacadabra"
    REPLACE ANY "ab" WITH "XY" IN rep
    PRINT #1, "REP"; "["; rep; "]"

    ' --- REMAIN$ negative Start ---
    remv = REMAIN$(-1, "hello world", "o")
    PRINT #1, "REMAIN"; "["; remv; "]"

    ' --- REGREPL capture groups ---
    rxtxt = "please email sales@powerbasic.com"
    rxpat = "email ([a-z]+)@([a-z.]+)"
    rxcap = "contact \01 at \00"
    REGREPL rxpat IN rxtxt WITH rxcap TO rxpos, rxres
    PRINT #1, "RXPOS"; rxpos
    PRINT #1, "RXRES"; "["; rxres; "]"

    ' --- ROUND ties-to-even (banker's) ---
    PRINT #1, "ROUND"; ROUND(0.5, 0); ROUND(1.5, 0); ROUND(2.5, 0); ROUND(2.345, 2)

    ' --- ROTATE width-aware ---
    bv = 221
    ROTATE RIGHT bv, 1
    PRINT #1, "ROR"; bv

    ' --- SHIFT signed right ---
    lv = &H80000000
    SHIFT SIGNED RIGHT lv, 1
    PRINT #1, "SHIFT"; lv

    ' --- SHRINK$ custom mask ---
    s = SHRINK$("a,,b", ",")
    PRINT #1, "SHRINK"; s

    ' --- SWITCH short-circuit: only the first true branch's value runs ---
    c_c1 = 0
    c_v1 = 0
    sw1 = SWITCH(cond1(), val1(), -1, 200)
    PRINT #1, "SWITCH"; sw1; c_c1; c_v1
    sws1 = SWITCH$(0, "aa", -1, "bb")
    PRINT #1, "SWITCHS"; sws1

    ' --- SPLIT WORD boundary ---
    SPLIT WORD "hello world foo", 9 TO sp1, sp2
    PRINT #1, "SPLIT"; "["; sp1; "]["; sp2; "]"

    ' --- STR$ digit control ---
    PRINT #1, "STR"; "["; STR$(5 / 3, 3); "]"

    ' --- SWAP numeric and string ---
    sw1 = 11
    sw2 = 22
    SWAP sw1, sw2
    PRINT #1, "SWAP"; sw1; sw2
    sws1 = "alpha"
    sws2 = "beta"
    SWAP sws1, sws2
    PRINT #1, "SWAPS"; "["; sws1; "]["; sws2; "]"

    ' --- SQR ---
    PRINT #1, "SQR"; SQR(16)

    ' --- RESOURCE$ missing id returns empty ---
    reslen = LEN(RESOURCE$(RCDATA, "999"))
    PRINT #1, "RES"; reslen

    CLOSE #1
END FUNCTION
