#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL f AS LONG
    LOCAL s AS STRING
    LOCAL i AS LONG

    f = FREEFILE
    OPEN "out283e.txt" FOR OUTPUT AS #f

    s = MKE$(1.5)
    PRINT #f, "LEN15="; LEN(s)
    FOR i = 1 TO LEN(s)
        PRINT #f, i; ":"; ASC(MID$(s, i, 1))
    NEXT i
    PRINT #f, "CVE15="; CVE(s, 1)

    s = MKE$(0)
    PRINT #f, "LEN0="; LEN(s)
    FOR i = 1 TO LEN(s)
        PRINT #f, i; ":"; ASC(MID$(s, i, 1))
    NEXT i
    PRINT #f, "CVE0="; CVE(s, 1)

    s = MKE$(-1.5)
    PRINT #f, "LENM="; LEN(s)
    FOR i = 1 TO LEN(s)
        PRINT #f, i; ":"; ASC(MID$(s, i, 1))
    NEXT i
    PRINT #f, "CVEM="; CVE(s, 1)

    CLOSE #f
END FUNCTION
