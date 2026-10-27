#COMPILE EXE
#DIM ALL
' --- batch 285: inline-ASM arithmetic on PB variables, dual-side.
' NOTE: the `or` instruction does NOT take effect with the installed
' clang/LLVM 19.44 (x86_64-pc-windows-msvc) - even plain C `y = y | 16`
' returns 600. ADD/AND/XOR/SHL/SHR all work. OR is therefore left out
' of this example; bitwise asm demo uses AND / XOR / SHL / SHR.
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

    ! MOV q, 1000000000000
    ! ADD q, 1
    ! SHR q, 1
    IF q = 500000000000 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s5 " + "q="; q

    ! AND x, &H0F
    IF x = 4 THEN pass = pass + 1 ELSE fail = fail + 1
    PRINT #1, "s6 " + "x="; x


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
