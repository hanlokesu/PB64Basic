#COMPILE EXE
#DIM ALL

GLOBAL g_calls AS LONG
GLOBAL g_total AS LONG

FUNCTION Bump() AS LONG
    INCR g_calls
    FUNCTION = g_calls
END FUNCTION

SUB AddTo(BYVAL n AS LONG)
    g_total = g_total + n
END SUB

FUNCTION ReadLater() AS LONG
    ' Bare zero-argument reference to a FUNCTION defined further down:
    ' PBWin accepts this spelling; PB64Basic used to demand parentheses
    ' or a prior declaration (batch 373).
    ReadLater = LaterValue
END FUNCTION

FUNCTION LaterValue() AS LONG
    LaterValue = 7
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL later AS LONG

    ' batch 373: a zero-argument call may carry a space before the empty
    ' parentheses.  PBWin 10 accepts `Bump ()`; PB64Basic used to reject it
    ' with "Unexpected token in expression: RParen".
    Bump ()
    Bump()

    ' The fix must not swallow a non-empty parenthesised first argument:
    ' `AddTo (5)` still passes 5 by value.
    AddTo (5)

    later = ReadLater()

    OPEN "batch373_out.txt" FOR OUTPUT AS #1
    PRINT #1, "calls="; g_calls; " total="; g_total; " later="; later
    CLOSE #1

    IF later <> 7 THEN FUNCTION = 1: EXIT FUNCTION
    IF g_calls <> 2 THEN FUNCTION = 1: EXIT FUNCTION
    IF g_total <> 5 THEN FUNCTION = 1: EXIT FUNCTION
    FUNCTION = 0
END FUNCTION