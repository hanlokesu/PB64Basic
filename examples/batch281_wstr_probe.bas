#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL ws  AS WSTRING
    LOCAL n   AS LONG
    LOCAL i   AS LONG
    LOCAL f   AS LONG
    LOCAL m   AS WSTRING
    LOCAL ws2 AS WSTRING
    LOCAL l   AS WSTRING
    LOCAL r   AS WSTRING
    LOCAL e   AS WSTRING

    f = FREEFILE
    OPEN "out281a.txt" FOR OUTPUT AS #f

    ws = "AB" + CHR$$(&H6F22) + CHR$$(&H5B57)
    PRINT #f, "LEN1="; LEN(ws)
    FOR i = 1 TO LEN(ws)
        PRINT #f, "ASC"; i; "="; ASC(ws, i)
    NEXT

    m = MID$(ws, 2, 2)
    PRINT #f, "MIDLEN="; LEN(m)
    FOR i = 1 TO LEN(m)
        PRINT #f, "MIDASC"; i; "="; ASC(m, i)
    NEXT

    PRINT #f, "INSTR1="; INSTR(ws, CHR$$(&H6F22))
    PRINT #f, "INSTR2="; INSTR(CHR$$(&H6F22), "AB" + CHR$$(&H6F22) + CHR$$(&H5B57))

    ws2 = ws + ws
    PRINT #f, "CONLEN="; LEN(ws2)

    l = LEFT$(ws, 2)
    PRINT #f, "LEFTLEN="; LEN(l)
    FOR i = 1 TO LEN(l)
        PRINT #f, "LEFTASC"; i; "="; ASC(l, i)
    NEXT

    r = RIGHT$(ws, 2)
    PRINT #f, "RIGHTLEN="; LEN(r)
    FOR i = 1 TO LEN(r)
        PRINT #f, "RIGHTASC"; i; "="; ASC(r, i)
    NEXT

    e = CHR$$(&H1F600)
    PRINT #f, "EMOJILEN="; LEN(e)
    FOR i = 1 TO LEN(e)
        PRINT #f, "EMOASC"; i; "="; ASC(e, i)
    NEXT

    CLOSE #f
END FUNCTION
