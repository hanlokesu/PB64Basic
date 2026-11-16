' batch 364: multi-line MACRO FUNCTION lowering (statement body + END MACRO = expression).
' The macro is a compile-time text substitution: the body statements are
' hoisted before the containing statement and the return expression replaces
' the call.  Covers parameters, MACROTEMP, EXIT MACRO, nested calls, and
' string- and numeric-returning macro functions.
#COMPILE EXE
#DIM ALL

GLOBAL g_last AS LONG

MACRO FUNCTION MF_ADD(a, b)
    MACROTEMP t
    DIM t AS LONG
    t = (a) + (b)
    IF t < 0 THEN EXIT MACRO
    t = t + 10
END MACRO = t

MACRO FUNCTION MF_SIDE(a)
    MACROTEMP st
    DIM st AS LONG
    st = (a)
    g_last = st
END MACRO = st

MACRO FUNCTION MF_STR(x)
    MACROTEMP s
    DIM s AS STRING
    s = x & "!"
    IF LEN(s) = 0 THEN EXIT MACRO
    s = s & "?"
END MACRO = s

MACRO FUNCTION MF_CONST()
    MACROTEMP c
    DIM c AS LONG
    c = 7
END MACRO = c

MACRO FUNCTION MF_ONE(x)
    MACROTEMP y
    DIM y AS LONG
    y = (x) + 1
END MACRO = y

MACRO WRAP(x) = MF_STR(x)

FUNCTION PBMAIN() AS LONG
    LOCAL fails AS LONG
    LOCAL n AS LONG
    LOCAL s AS STRING

    n = MF_ADD(2, 3)
    IF n <> 15 THEN fails = fails + 1

    n = MF_SIDE(5)
    IF n <> 5 THEN fails = fails + 1
    IF g_last <> 5 THEN fails = fails + 1

    s = MF_STR("ok")
    IF s <> "ok!?" THEN fails = fails + 1

    n = MF_ADD(4, 5) + MF_ADD(1, 1)
    IF n <> 31 THEN fails = fails + 1

    n = MF_CONST
    IF n <> 7 THEN fails = fails + 1

    n = MF_ONE(41)
    IF n <> 42 THEN fails = fails + 1

    s = WRAP("mix")
    IF s <> "mix!?" THEN fails = fails + 1

    OPEN "batch364_out.txt" FOR OUTPUT AS #1
    PRINT #1, "n="; n; " s="; s; " last="; g_last; " fails="; fails
    CLOSE #1

    IF fails <> 0 THEN FUNCTION = 1 ELSE FUNCTION = 0
END FUNCTION
