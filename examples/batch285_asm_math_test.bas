#COMPILE EXE
#DIM ALL
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- batch 285: inline-ASM arithmetic on PB variables, dual-side.
' NOTE (2026-10-27 resolution): the earlier "or does not take effect"
' report was a WRONG EXPECTATION: 600 | 16 == 600 because bit 4 is already
' set in 600 (600 = 0x258).  Real-change samples (600 | 0x100 = 856,
' 4 | 0x80 = 132) work on both PBWin10 and PB64Basic.  OR is included again.
'     The same ! lines compile on PBWin10 (32-bit, variables as symbols)
'     and on PB64Basic (64-bit LLVM inline asm, variables as memory
'     operands).  Every step is written to a probe file and compared.
FUNCTION PBMAIN() AS LONG
    LOCAL x AS LONG
    LOCAL y AS LONG
    LOCAL w AS WORD
    LOCAL b AS BYTE
    LOCAL q AS QUAD
    LOCAL pass AS LONG
    LOCAL fail AS LONG
    pass = 0
    fail = 0
    OPEN "out285math.txt" FOR OUTPUT AS #1

    x = 100
    ! MOV y, x
    ! ADD y, 23
    ! SUB y, 3
    IF y = 120 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s1 " + "y="; y

    ! MOV EAX, y
    ! IMUL EAX, 5
    ! MOV y, EAX
    IF y = 600 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s2 " + "y="; y

    ! MOV w, 30000
    ! ADD w, 1000
    IF w = 31000 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s3 " + "w="; w

    ! MOV b, 200
    ! ADD b, 50
    IF b = 250 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s4 " + "b="; b

    #IF %MY_PBVER = 10
    q = 1000000000000
    q = q + 1
    q = q \ 2
    #ELSE
    q = 1000000000000
    ! ADD q, 1
    ! SHR q, 1
    #ENDIF
    IF q = 500000000000 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s5 " + "q="; q

    ! AND x, &H0F
    IF x = 4 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s6 " + "x="; x


    ! OR x, &H80
    IF x = 132 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s7 " + "x="; x

    ! XOR w, &HFFFF
    IF w = 34535 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s8 " + "w="; w

    ! SHL b, 1
    IF b = 244 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s9 " + "b="; b

    ' register round-trip through a PB variable
    ! MOV EAX, 42
    ! MOV x, EAX
    IF x = 42 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s10 " + "x="; x

    PRINT #1, "y="; y; " w="; w; " b="; b; " q="; q; " x="; x
    PRINT #1, "asm math: pass="; pass; " fail="; fail
    CLOSE #1
END FUNCTION
