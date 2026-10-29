' batch303v_test.bas - VARIANT$(BYTE, vrntvar) second signature (legacy #4).
' CHW VARIANT_function1.htm: the BYTE option tells PB the variant holds a
' set of BYTES (e.g. a UDT stored via LET v = udt AS STRING) rather than
' wide Unicode words; the result is always the ANSI byte string, with no
' WIDE->ANSI conversion. Dual-run vs PBWin.
#COMPILE EXE
#DIM ALL
FUNCTION PBMAIN() AS LONG
    LOCAL v AS VARIANT
    LOCAL s AS STRING
    OPEN "batch303v_out.txt" FOR OUTPUT AS #1
    v = "hello"
    s = VARIANT$(BYTE, v)
    PRINT #1, "bytestr="; s
    s = VARIANT$(v)
    PRINT #1, "normstr="; s
    CLOSE #1
    FUNCTION = 0
END FUNCTION
