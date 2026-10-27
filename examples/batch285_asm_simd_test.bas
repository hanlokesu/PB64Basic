#COMPILE EXE
#DIM ALL

' --- batch 285: MMX / SSE / x87 inline asm.
'     Same 4 groups run verbatim on both compilers (PBWin10 32-bit and
'     PB64Basic 64-bit); output goes to out285simd.txt for side-by-side diff.
FUNCTION PBMAIN() AS LONG
    LOCAL q_in AS QUAD
    LOCAL q_out AS QUAD
    LOCAL d1 AS DOUBLE
    LOCAL d2 AS DOUBLE
    LOCAL pass AS LONG
    LOCAL fail AS LONG
    pass = 0
    fail = 0
    OPEN "out285simd.txt" FOR OUTPUT AS #1

    ' MMX: load a QUAD into mm0, add-packed bytes, store back
    q_in = 33752069
    q_out = 0
    ! MOVQ MM0, q_in
    ! MOVQ MM1, q_in
    ! PADDB MM0, MM1
    ! MOVQ q_out, MM0
    ! EMMS
    IF q_out = 67504138 THEN pass = pass + 1 ELSE fail = fail + 1

    ' x87: FLD1 then FSTP into a DOUBLE
    d1 = 0
    d2 = 0
    ! FLD1
    ! FSTP d1
    IF d1 > 0.9 AND d1 < 1.1 THEN pass = pass + 1 ELSE fail = fail + 1

    ' x87: push two constants, FADD (register form), store
    ! FLD1
    ! FLD1
    ! FADD ST(0), ST(1)
    ! FSTP d2
    IF d2 > 1.9 AND d2 < 2.1 THEN pass = pass + 1 ELSE fail = fail + 1

    ' SSE: zero a register pair (compile + run proof)
    ! XORPS XMM0, XMM0
    ! XORPS XMM1, XMM1
    ! ADDPS XMM1, XMM0
    pass = pass + 1

    PRINT #1, "asm simd: pass="; pass; " fail="; fail
    CLOSE #1
END FUNCTION
