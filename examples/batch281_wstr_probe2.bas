#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL e2  AS WSTRING
    LOCAL i   AS LONG
    LOCAL f   AS LONG
    LOCAL s   AS STRING
    LOCAL ws  AS WSTRING
    LOCAL m   AS WSTRING

    f = FREEFILE
    OPEN "out281b.txt" FOR OUTPUT AS #f

    ' ANSI literal INSTR (both STRING) - DBCS aware?
    PRINT #f, "A1="; INSTR("ABh×Ö", "h")

    ' surrogate pair via two CHR$$ BMP chars
    e2 = CHR$$(&HD83D) + CHR$$(&HDE00)
    PRINT #f, "SURLEN="; LEN(e2)
    FOR i = 1 TO LEN(e2)
        PRINT #f, "SURASC"; i; "="; ASC(e2, i)
    NEXT

    ' mixed STRING + WSTRING concat -> result type?
    s = "X"
    ws = "YZ"
    m = s + ws
    PRINT #f, "MIXLEN="; LEN(m)
    FOR i = 1 TO LEN(m)
        PRINT #f, "MIXASC"; i; "="; ASC(m, i)
    NEXT

    ' WSTRING from STRING var assignment (ANSI -> wide auto?)
    ws = "ABC"
    PRINT #f, "WSLEN="; LEN(ws)
    FOR i = 1 TO LEN(ws)
        PRINT #f, "WSASC"; i; "="; ASC(ws, i)
    NEXT

    ' negative positions in MID$ on WSTRING
    m = MID$(ws, -1, 2)
    PRINT #f, "NEGLEN="; LEN(m)
    FOR i = 1 TO LEN(m)
        PRINT #f, "NEGASC"; i; "="; ASC(m, i)
    NEXT

    CLOSE #f
END FUNCTION
