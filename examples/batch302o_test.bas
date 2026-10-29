' batch302o_test.bas - CLIPBOARD GET OEMTEXT high-byte (0x80+) ANSI->OEM fallback
' Legacy #9: batch262 verified ASCII passthrough only; whether PBWin maps
' 0x80+ ANSI bytes to OEM (and whether fork's raw ANSI passthrough matches)
' was never measured. Bytes are chosen across the high range and written raw
' (no NUL) so the output file can be compared byte-for-byte.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL t AS STRING
    LOCAL r, i AS LONG
    LOCAL s AS STRING
    OPEN "batch302o_out.txt" FOR OUTPUT AS #1
    s = CHR$(128) & CHR$(145) & CHR$(169) & CHR$(200) & CHR$(233) & CHR$(255)
    FOR i = 1 TO 5
        CLIPBOARD RESET
        CLIPBOARD SET TEXT s TO r
        t = ""
        CLIPBOARD GET OEMTEXT TO t, r
        IF LEN(t) = LEN(s) AND r = -1 THEN EXIT FOR
    NEXT i
    PRINT #1, "oemlen:" + STR$(LEN(t)) + ":r:" + STR$(r)
    PRINT #1, "oembytes:"
    PRINT #1, t
    CLOSE #1
    FUNCTION = 0
END FUNCTION
