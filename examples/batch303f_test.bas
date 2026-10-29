' batch303f_test.bas - legacy #7: TYPE FixedString members as INPUT# /
' LINE INPUT# targets (WRITE# -> INPUT#; PRINT# -> LINE INPUT#).  A 14-char
' string into STRING*8 must truncate to 7 chars + NUL, and the LONG member
' that follows must keep its value (no neighbour overwrite).
' Names avoid PBWin Error 466 reserved-name clashes.
#COMPILE EXE
#DIM ALL
TYPE Rec303
  fs AS STRING * 8
  nn AS LONG
END TYPE
FUNCTION PBMAIN() AS LONG
    LOCAL rec AS Rec303
    OPEN "batch303f_in.txt" FOR OUTPUT AS #1
    WRITE #1, "abcdefghijklmnop", 123
    CLOSE #1
    OPEN "batch303f_in.txt" FOR INPUT AS #1
    INPUT #1, rec.fs, rec.nn
    CLOSE #1
    OPEN "batch303f_out.txt" FOR OUTPUT AS #2
    PRINT #2, "A:" & rec.fs & "|" & STR$(rec.nn)
    CLOSE #2
    OPEN "batch303f_in.txt" FOR OUTPUT AS #3
    PRINT #3, "line-one"
    CLOSE #3
    OPEN "batch303f_in.txt" FOR INPUT AS #3
    LINE INPUT #3, rec.fs
    CLOSE #3
    OPEN "batch303f_out.txt" FOR APPEND AS #2
    PRINT #2, "B:" & rec.fs & "|" & STR$(rec.nn)
    CLOSE #2
    FUNCTION = 0
END FUNCTION
