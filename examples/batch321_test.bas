#COMPILE EXE
#DIM ALL

TYPE PQRec
    a AS LONG
    b AS LONG
END TYPE

SUB Check(BYVAL cond AS LONG, BYVAL label AS STRING, BYREF nfail AS LONG)
    IF cond = 0 THEN
        nfail = nfail + 1
        PRINT "FAIL: "; label
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    DIM pa(1 TO 6) AS STRING
    LOCAL fs AS STRING
    DIM buf(0 TO 31) AS BYTE
    LOCAL memaddr AS QUAD
    LOCAL b8 AS LONG
    LOCAL w16 AS LONG
    LOCAL d32 AS DWORD
    LOCAL q64 AS QUAD
    LOCAL s8 AS STRING
    LOCAL sz AS STRING
    LOCAL ptv AS PQRec
    LOCAL fn AS LONG
    LOCAL lval AS LONG
    LOCAL qval AS QUAD
    LOCAL rval AS STRING
    LOCAL wrval AS STRING
    LOCAL qbig AS QUAD
    LOCAL pri AS LONG
    LOCAL valid AS LONG
    LOCAL bogus AS STRING
    LOCAL pc AS LONG
    LOCAL nfail AS LONG

    ' === PARSE statement ===
    pa(1) = "": pa(2) = "": pa(3) = ""
    PARSE "apple,banana,cherry", pa()
    Check (pa(1) = "apple" AND pa(2) = "banana" AND pa(3) = "cherry"), "PARSE comma", nfail

    pa(1) = "": pa(2) = "": pa(3) = ""
    PARSE "a;b,c", pa(), ANY ";,"
    Check (pa(1) = "a" AND pa(2) = "b" AND pa(3) = "c"), "PARSE ANY", nfail

    pa(1) = "": pa(2) = "": pa(3) = ""
    PARSE "x-y-z", pa(), "-"
    Check (pa(1) = "x" AND pa(2) = "y" AND pa(3) = "z"), "PARSE explicit", nfail

    ' === PARSE$ function ===
    Check (PARSE$("apple,banana,cherry", 2) = "banana"), "PARSE$ pos", nfail
    Check (PARSE$("apple,banana,cherry", -1) = "cherry"), "PARSE$ neg1", nfail
    Check (PARSE$("apple,banana,cherry", -2) = "banana"), "PARSE$ neg2", nfail
    Check (PARSE$("apple,banana,cherry", 0) = ""), "PARSE$ zero", nfail
    Check (PARSE$("apple,banana,cherry", 99) = ""), "PARSE$ out of range", nfail
    Check (PARSE$("", 1) = ""), "PARSE$ empty", nfail
    Check (PARSE$("abcdef", 1) = "abcdef"), "PARSE$ no delimiter", nfail
    Check (PARSE$("x-y-z", "-", 3) = "z"), "PARSE$ delimiter", nfail
    Check (PARSE$("a;b,c", ANY ";,", 3) = "c"), "PARSE$ ANY", nfail

    ' === PARSECOUNT function ===
    Check (PARSECOUNT("one,two,three") = 3), "PARSECOUNT three", nfail
    Check (PARSECOUNT("") = 1), "PARSECOUNT empty", nfail
    Check (PARSECOUNT("abcdef") = 1), "PARSECOUNT no delimiter", nfail
    Check (PARSECOUNT("1;2,3", ANY ",;") = 3), "PARSECOUNT ANY", nfail
    Check (PARSECOUNT("xx1x", "x") = 4), "PARSECOUNT x", nfail

    ' === PATHNAME$ (text-only parse) ===
    fs = "C:\dir\sub\file.txt"
    Check (PATHNAME$(FULL, fs) = "C:\dir\sub\file.txt"), "PATHNAME FULL", nfail
    Check (PATHNAME$(PATH, fs) = "C:\dir\sub\"), "PATHNAME PATH", nfail
    Check (PATHNAME$(NAME, fs) = "file"), "PATHNAME NAME", nfail
    Check (PATHNAME$(EXTN, fs) = ".txt"), "PATHNAME EXTN", nfail
    Check (PATHNAME$(NAMEX, fs) = "file.txt"), "PATHNAME NAMEX", nfail

    ' === PATHSCAN$ (real disk scan) ===
    OPEN "pqscan.txt" FOR OUTPUT AS #2
    PRINT #2, "scan target"
    CLOSE #2
    Check (PATHSCAN$(NAME, "pqscan.txt") = "pqscan"), "PATHSCAN NAME", nfail
    Check (PATHSCAN$(EXTN, "pqscan.txt") = ".txt"), "PATHSCAN EXTN", nfail
    Check (PATHSCAN$(NAMEX, "pqscan.txt") = "pqscan.txt"), "PATHSCAN NAMEX", nfail
    Check (PATHSCAN$(NAMEX, "no_such_file_xyz.txt") = ""), "PATHSCAN missing", nfail
    KILL "pqscan.txt"

    ' === PEEK numeric ===
    memaddr = VARPTR(buf(0))
    POKE BYTE, memaddr, 65
    b8 = PEEK(memaddr)
    Check (b8 = 65), "PEEK byte", nfail
    POKE WORD, memaddr, &H1234
    w16 = PEEK(WORD, memaddr)
    Check (w16 = &H1234), "PEEK word", nfail
    POKE DWORD, memaddr, &H12345678
    d32 = PEEK(DWORD, memaddr)
    Check (d32 = &H12345678), "PEEK dword", nfail
    POKE QUAD, memaddr, 987654321012345
    q64 = PEEK(QUAD, memaddr)
    Check (q64 = 987654321012345), "PEEK quad", nfail

    ' === POKE multi-value ===
    POKE BYTE, memaddr, 7, 8, 9
    Check (PEEK(memaddr) = 7 AND PEEK(memaddr + 1) = 8 AND PEEK(memaddr + 2) = 9), "POKE multi", nfail

    ' === PEEK$ / POKE$ (raw bytes) ===
    POKE$ memaddr, "ABCDEF"
    s8 = PEEK$(memaddr, 6)
    Check (s8 = "ABCDEF"), "PEEK$ raw", nfail

    ' === POKE$ STRINGZ / PEEK$ STRINGZ (NUL-terminated) ===
    POKE$ STRINGZ, memaddr, "HI"
    sz = PEEK$(STRINGZ, memaddr, 8)
    Check (sz = "HI" AND LEN(sz) = 2), "PEEK$ stringz", nfail

    ' === PREFIX / END PREFIX (UDT member form) ===
    PREFIX "ptv."
        a = 10
        b = 20
    END PREFIX
    Check (ptv.a = 10 AND ptv.b = 20), "PREFIX", nfail

    ' === PUT / GET binary with an empty position (seek fix) ===
    fn = FREEFILE
    OPEN "pqbin.dat" FOR BINARY AS #fn
    lval = 123456789
    qval = 987654321012345
    PUT #fn, , lval
    PUT #fn, , qval
    CLOSE #fn
    lval = 0
    qval = 0
    OPEN "pqbin.dat" FOR BINARY AS #fn
    GET #fn, , lval
    GET #fn, , qval
    CLOSE #fn
    Check (lval = 123456789 AND qval = 987654321012345), "PUT/GET empty position", nfail
    KILL "pqbin.dat"

    ' === PUT$ / GET$ (ANSI bytes) ===
    fn = FREEFILE
    OPEN "pqansi.dat" FOR BINARY AS #fn
    PUT$ #fn, "ANSI text"
    CLOSE #fn
    OPEN "pqansi.dat" FOR BINARY AS #fn
    GET$ #fn, 9, rval
    CLOSE #fn
    Check (rval = "ANSI text"), "PUT$/GET$", nfail
    KILL "pqansi.dat"

    ' === PUT$$ / GET$$ (WIDE chars) ===
    fn = FREEFILE
    OPEN "pqwide.dat" FOR BINARY AS #fn
    PUT$$ #fn, "hello"
    Check (LOF(fn) = 10), "PUT$$ length", nfail
    CLOSE #fn
    OPEN "pqwide.dat" FOR BINARY AS #fn
    GET$$ #fn, 5, wrval
    CLOSE #fn
    Check (wrval = "hello"), "GET$$", nfail
    KILL "pqwide.dat"

    ' === QUAD data type (64-bit integer) ===
    qbig = 987654321012345
    Check (STR$(qbig) = " 987654321012345"), "QUAD STR$", nfail
    Check (qbig + 1 = 987654321012346), "QUAD add", nfail
    Check (qbig * 2 = 1975308642024690), "QUAD multiply", nfail

    ' === PROCESS GET/SET PRIORITY (safe round trip) ===
    PROCESS GET PRIORITY TO pri
    valid = 0
    IF pri = &H20 OR pri = &H40 OR pri = &H80 OR pri = &H100 THEN valid = 1
    Check (valid = 1), "PROCESS GET PRIORITY", nfail
    PROCESS SET PRIORITY pri
    PROCESS GET PRIORITY TO pri
    valid = 0
    IF pri = &H20 OR pri = &H40 OR pri = &H80 OR pri = &H100 THEN valid = 1
    Check (valid = 1), "PROCESS round trip", nfail

    ' === PROGID$ negative lookup (bogus -> empty) ===
    bogus = "1234567890abcdef"
    Check (LEN(PROGID$(bogus)) = 0), "PROGID$ negative", nfail

    ' === PRINTER$ out-of-range (empty) and PRINTERCOUNT bounded ===
    Check (PRINTER$(NAME, 99999) = ""), "PRINTER$ out of range", nfail
    pc = PRINTERCOUNT
    Check (pc >= 0), "PRINTERCOUNT bounded", nfail

    ' === PLAY WAVE END (safe stop when idle) ===
    PLAY WAVE END

    IF nfail = 0 THEN PRINT "batch321 ok"
    FUNCTION = nfail
END FUNCTION
