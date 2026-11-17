' batch379_ansi_eof.bas - ANSI sequential DOS EOF marker (CHR$(26)).
' CHW sequential text layer treats 0x1A as end-of-file; the ANSI reader must
' match the WIDE reader, not return the marker as data.
#COMPILE EXE
FUNCTION PBMAIN() AS LONG
    LOCAL fails AS LONG
    LOCAL s AS STRING
    LOCAL e AS LONG

    OPEN "batch379_ansi_eof.dat" FOR OUTPUT AS #1
    PRINT #1, "A"; CHR$(26); "B"
    CLOSE #1

    OPEN "batch379_ansi_eof.dat" FOR INPUT AS #1
    LINE INPUT #1, s
    IF s <> "A" THEN INCR fails
    e = EOF(#1)
    IF e <> -1 THEN INCR fails
    CLOSE #1

    OPEN "batch379_ansi_eof.txt" FOR OUTPUT AS #1
    PRINT #1, "batch379 ansi eof, failures="; fails
    PRINT #1, "s=["; s; "] e="; e
    CLOSE #1
    IF fails <> 0 THEN FUNCTION = 1 ELSE FUNCTION = 0
END FUNCTION
