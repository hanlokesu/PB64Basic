' batch276b probe: UNION + ENUM (SINGULAR) - PBWin vs fork dual run
' Output file: b276b_out.txt (PRINT #f, single-expression form, PBWin rule)
#COMPILE EXE
UNION VarConvert
    iLong AS LONG
    iDword AS DWORD
    sStr AS STRING * 4
END UNION
ENUM abc SINGULAR
    one = 1
    two
    three
END ENUM
FUNCTION PBMAIN() AS LONG
    LOCAL f AS LONG
    f = FREEFILE
    OPEN "b276b_out.txt" FOR OUTPUT AS #f
    DIM x AS VarConvert
    x.iLong = 123456
    PRINT #f, "ilong=" + STR$(x.iLong)
    PRINT #f, "idword=" + STR$(x.iDword)
    PRINT #f, "one=" + STR$(%one)
    PRINT #f, "two=" + STR$(%two)
    PRINT #f, "three=" + STR$(%three)
    CLOSE #f
    FUNCTION = 0
END FUNCTION
