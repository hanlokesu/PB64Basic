' batch295_variant_test.bas - VARIANT#(vrntvar) + VARIANTVT(vrntvar) (CHW
' VARIANT_function / VARIANTVT_function).  VARIANTVT returns the internal VT
' data type stored in a Variant variable (equates: 0 VT_EMPTY, 2 VT_I2,
' 3 VT_I4, 4 VT_R4, 5 VT_R8, 8 VT_BSTR, ...).  VARIANT# returns the numeric
' value regardless of the internal numeric representation.
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL v AS VARIANT
    LOCAL a AS LONG, d AS DOUBLE
    OPEN "batch295_out.txt" FOR OUTPUT AS #1
    v = 999&
    PRINT #1, "vt_i4="; VARIANTVT(v)
    a = VARIANT#(v)
    PRINT #1, "num_i4="; a
    v = 1.5!
    PRINT #1, "vt_r4="; VARIANTVT(v)
    d = VARIANT#(v)
    PRINT #1, "num_r4="; d
    v = 2.75#
    PRINT #1, "vt_r8="; VARIANTVT(v)
    d = VARIANT#(v)
    PRINT #1, "num_r8="; d
    v = "hello"
    PRINT #1, "vt_str="; VARIANTVT(v)
    v = 0
    PRINT #1, "vt_zero="; VARIANTVT(v)
    CLOSE #1
    FUNCTION = 0
END FUNCTION
