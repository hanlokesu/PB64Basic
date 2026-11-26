#COMPILE EXE
#DIM ALL

' batch 409 regression: José Roca include-file syntax used by gbClientCapture v2.1.
' Covers `@ptr.@pointer_field`, reserved-word method names (Xor/On), and the
' MMSystem.inc MAKEFOURCC macro that must not rewrite a FUNCTION declaration.

TYPE InnerT
    MimeType AS STRING
    Code AS LONG
END TYPE

TYPE OuterT
    pInner AS InnerT PTR
END TYPE

CLASS Bits
    INSTANCE v AS LONG
    METHOD Xor(n AS LONG) AS LONG
        v = v XOR n
        FUNCTION = v
    END METHOD
    METHOD On(n AS LONG) AS LONG
        FUNCTION = n
    END METHOD
END CLASS

FUNCTION PBMAIN() AS LONG
    LOCAL o AS OuterT
    LOCAL i AS InnerT
    LOCAL op AS OuterT PTR
    LOCAL b AS Bits
    LOCAL fails AS LONG

    i.MimeType = "image/png"
    i.Code = 42
    o.pInner = VARPTR(i)
    op = VARPTR(o)

    IF @op.@pInner.MimeType <> "image/png" THEN fails = fails + 1
    IF @op.@pInner.Code <> 42 THEN fails = fails + 1

    b.v = 0
    IF Bits_Xor(b, 5) <> 5 THEN fails = fails + 1
    IF Bits_Xor(b, 3) <> 6 THEN fails = fails + 1
    IF Bits_On(b, 7) <> 7 THEN fails = fails + 1

    PRINT "FAILURES="; fails
    FUNCTION = fails
END FUNCTION
