' batch317_no_test.bas - N/O family (batch 317) double-run probe.
' Headless: writes results to batch317_out.txt via PRINT #1. No MSGBOX,
' no WAITKEY$, no DIALOG, no INPUT.  Semicolon item separators are used (the
' fork appends one space after each PRINT # item, matching PBWin semicolon
' semantics; comma tab-zones are a pre-existing fork gap, so commas are avoided
' here).  Values only, so output is byte-for-byte comparable with PBWin10.
#COMPILE EXE
#DIM ALL

FUNCTION PBMAIN()
    OPEN "batch317_out.txt" FOR OUTPUT AS #1

    ' --- OCT / OCT$ (integer -> octal) ---
    PRINT #1, "OCT"; OCT$(8); OCT$(10); OCT$(4095); OCT$(12345)

    ' --- NOT integer truth table ---
    PRINT #1, "NOT0"; NOT 0
    PRINT #1, "NOT-1"; NOT -1
    PRINT #1, "NOT1"; NOT 1
    PRINT #1, "NOT16383"; NOT 16383

    ' --- NOT float: PBWin rounds half-to-even then bitwise (measured) ---
    PRINT #1, "NOT1.5"; NOT 1.5
    PRINT #1, "NOT2.5"; NOT 2.5
    PRINT #1, "NOT3.5"; NOT 3.5

    ' --- OR / AND / XOR float: round half-to-even then bitwise ---
    PRINT #1, "OR1.5"; 1.5 OR 2.0
    PRINT #1, "OR2.5"; 2.5 OR 1.0
    PRINT #1, "AND2.5"; 2.5 AND 3.5
    PRINT #1, "XOR2.5"; 2.5 XOR 1.0

    ' --- OEMTOCHR$ ANSI + control-code pass-through ---
    PRINT #1, "OEM"; LEN(OEMTOCHR$("A")); ASC(OEMTOCHR$("A"))
    PRINT #1, "OEMctl"; ASC(OEMTOCHR$(CHR$(1)))
    DIM w AS WSTRING
    w = OEMTOCHR$(GRAPHIC, "A")
    PRINT #1, "OEMW"; LEN(w); ASC(w)
    w = OEMTOCHR$(GRAPHIC, CHR$(1))
    PRINT #1, "OEMWctl"; ASC(w)

    ' --- NUL$ (null string) ---
    PRINT #1, "NUL"; LEN(NUL$(0))

    ' --- OPEN omitted FOR (default RANDOM): open/close only.  PRINT to a
    ' RANDOM-mode file raises error 54 on PBWin, so do not write to it here.
    OPEN "batch317_rnd.tmp" AS #2
    CLOSE #2

    ' --- NAME: success, missing source (53), dest-exists (58) ---
    DIM e1 AS LONG, e2 AS LONG, e3 AS LONG
    OPEN "batch317_src.tmp" FOR OUTPUT AS #2
    PRINT #2, "x"
    CLOSE #2
    OPEN "batch317_dst_existing.tmp" FOR OUTPUT AS #3
    PRINT #3, "y"
    CLOSE #3
    ON ERROR RESUME NEXT
    NAME "batch317_src.tmp" AS "batch317_newname.tmp"
    e1 = ERR
    NAME "batch317_nonexistent.tmp" AS "batch317_z.tmp"
    e2 = ERR
    NAME "batch317_newname.tmp" AS "batch317_dst_existing.tmp"
    e3 = ERR
    PRINT #1, "NAME"; e1; e2; e3

    ' --- OBJRESULT$ bare (no prior COM call => hr=0 => system text) ---
    PRINT #1, "OBJRES"; OBJRESULT$()

    CLOSE #1
END FUNCTION
