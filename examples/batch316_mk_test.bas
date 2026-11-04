#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

#COMPILE EXE
#DIM ALL

' batch316 - MK* numeric-to-string family (17 entries: 11 CHW $ forms + 6 fork-only non-$).
' The 11 $ forms are dumped to mout.txt identically on PBWin10 and PB64Basic.
' The 6 non-$ forms (MKBYT/MKCUX/MKDWD/MKE/MKS/MKWRD) are fork-only builtins that
' alias their $ sibling via normalize_name stripping a trailing $; they are
' self-checked byte-for-byte against the $ sibling inside the fork branch only.

DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB

' Shared dump of the 11 CHW-documented $ forms -> identical mout.txt on both compilers.
SUB DumpDollar(BYVAL f AS LONG)
    LOCAL s AS STRING, i AS LONG, b AS STRING, outl AS STRING
    s = MKBYT$(255)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "BYT len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVBYT(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKI$(-2)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "I len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVI(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKWRD$(65535)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "WRD len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVWRD(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKL$(-2)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "L len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVL(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKDWD$(4294967295#)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "DWD len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVDWD(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKQ$(-2)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "Q len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVQ(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKS$(1.5)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "S len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVS(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKD$(1.5)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "D len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVD(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKCUR$(123.45)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "CUR len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVCUR(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKCUX$(123.45)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "CUX len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVCUX(s, 1))) + " bytes=" + b
    PRINT #f, outl
    s = MKE$(1.5)
    b = ""
    FOR i = 1 TO LEN(s)
        b = b + TRIM$(STR$(ASC(MID$(s, i, 1)))) + ","
    NEXT i
    outl = "E len=" + TRIM$(STR$(LEN(s))) + " cv=" + TRIM$(STR$(CVE(s, 1))) + " bytes=" + b
    PRINT #f, outl
END SUB

#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    LOCAL f AS LONG
    f = FREEFILE
    OPEN "mout.txt" FOR OUTPUT AS #f
    DumpDollar f
    CLOSE #f
END FUNCTION
#ELSE
FUNCTION PBMAIN() AS LONG
    LOCAL f AS LONG, nfail AS LONG
    LOCAL s AS STRING, sN AS STRING, sD AS STRING
    nfail = 0
    f = FREEFILE
    OPEN "mout.txt" FOR OUTPUT AS #f
    DumpDollar f
    CLOSE #f

    s = MKBYT$(255) : IF LEN(s) <> 1 OR CVBYT(s, 1) <> 255 THEN nfail = nfail + 1
    s = MKI$(-2) : IF LEN(s) <> 2 OR CVI(s, 1) <> -2 THEN nfail = nfail + 1
    s = MKWRD$(65535) : IF LEN(s) <> 2 OR CVWRD(s, 1) <> 65535 THEN nfail = nfail + 1
    s = MKL$(-2) : IF LEN(s) <> 4 OR CVL(s, 1) <> -2 THEN nfail = nfail + 1
    s = MKDWD$(4294967295#) : IF LEN(s) <> 4 OR CVDWD(s, 1) <> 4294967295 THEN nfail = nfail + 1
    s = MKQ$(-2) : IF LEN(s) <> 8 OR CVQ(s, 1) <> -2 THEN nfail = nfail + 1
    s = MKS$(1.5) : IF LEN(s) <> 4 OR CVS(s, 1) <> 1.5 THEN nfail = nfail + 1
    s = MKD$(1.5) : IF LEN(s) <> 8 OR CVD(s, 1) <> 1.5 THEN nfail = nfail + 1
    s = MKCUR$(123.45) : IF LEN(s) <> 8 OR CVCUR(s, 1) <> 123.45# THEN nfail = nfail + 1
    s = MKCUX$(123.45) : IF LEN(s) <> 8 OR CVCUX(s, 1) <> 123.45# THEN nfail = nfail + 1
    s = MKE$(1.5) : IF LEN(s) <> 10 OR CVE(s, 1) <> 1.5 THEN nfail = nfail + 1

    sN = MKBYT(255) : sD = MKBYT$(255)
    IF LEN(sN) <> 1 OR sN <> sD OR CVBYT(sN, 1) <> 255 THEN nfail = nfail + 1
    sN = MKCUX(123.45) : sD = MKCUX$(123.45)
    IF LEN(sN) <> 8 OR sN <> sD OR CVCUX(sN, 1) <> 123.45# THEN nfail = nfail + 1
    sN = MKDWD(4294967295#) : sD = MKDWD$(4294967295#)
    IF LEN(sN) <> 4 OR sN <> sD OR CVDWD(sN, 1) <> 4294967295 THEN nfail = nfail + 1
    sN = MKE(1.5) : sD = MKE$(1.5)
    IF LEN(sN) <> 10 OR sN <> sD OR CVE(sN, 1) <> 1.5 THEN nfail = nfail + 1
    sN = MKS(1.5) : sD = MKS$(1.5)
    IF LEN(sN) <> 4 OR sN <> sD OR CVS(sN, 1) <> 1.5 THEN nfail = nfail + 1
    sN = MKWRD(65535) : sD = MKWRD$(65535)
    IF LEN(sN) <> 2 OR sN <> sD OR CVWRD(sN, 1) <> 65535 THEN nfail = nfail + 1

    ConPrint "=== FAILURES:" & TRIM$(STR$(nfail)) & " ==="
    IF nfail = 0 THEN ConPrint "batch316 mk ok"
    FUNCTION = nfail
END FUNCTION
#ENDIF
