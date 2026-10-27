#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN() AS LONG
    LOCAL f AS LONG
    LOCAL s AS STRING

    f = FREEFILE
    OPEN "out283.txt" FOR OUTPUT AS #f

    s = MKBYT$(255)
    PRINT #f, "BYT="; LEN(s); CVBYT(s, 1)
    s = MKI$(-2)
    PRINT #f, "I="; LEN(s); CVI(s, 1)
    s = MKWRD$(65535)
    PRINT #f, "WRD="; LEN(s); CVWRD(s, 1)
    s = MKL$(-2)
    PRINT #f, "L="; LEN(s); CVL(s, 1)
    s = MKDWD$(4294967295#)
    PRINT #f, "DWD="; LEN(s); CVDWD(s, 1)
    s = MKQ$(-2)
    PRINT #f, "Q="; LEN(s); CVQ(s, 1)
    s = MKS$(1.5)
    PRINT #f, "S="; LEN(s); CVS(s, 1)
    s = MKD$(1.5)
    PRINT #f, "D="; LEN(s); CVD(s, 1)
    s = MKCUR$(123.45)
    PRINT #f, "CUR="; LEN(s); CVCUR(s, 1)
    s = MKCUX$(123.45)
    PRINT #f, "CUX="; LEN(s); CVCUX(s, 1)
    s = MKE$(1.5)
    PRINT #f, "E="; LEN(s); CVE(s, 1)

    CLOSE #f
END FUNCTION
