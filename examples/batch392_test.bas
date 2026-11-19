' batch392_test.bas - WinAPI x64 structure layout and constant expressions.
' Covers WSTRINGZ * %MAX_PATH, QUAD/QWORD alignment, TYPE ... NATIVE,
' parenthesized/bitwise %CONSTANT values, SIZEOF(TYPENAME), and the fork-only
' %PB64_TARGET_64 %DEF marker on both x64 and i686 targets.
#COMPILE EXE
#DIM ALL

%MAX_PATH = 260
%B392_A = 1
%B392_B = 2
%B392_OR = (%B392_A OR %B392_B)
%B392_SYNC = (&H00100000)

TYPE B392W
    A AS BYTE
    S AS WSTRINGZ * %MAX_PATH
    B AS LONG
END TYPE

TYPE B392Q QWORD
    A AS BYTE
    B AS LONG
END TYPE

TYPE B392QUAD QUAD
    A AS BYTE
    B AS LONG
END TYPE

TYPE B392N NATIVE
    A AS BYTE
    B AS LONG
    C AS QUAD
END TYPE

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL target64 AS LONG
    LOCAL ver AS STRING
    LOCAL w AS B392W
    LOCAL q AS B392Q
    LOCAL uq AS B392QUAD
    LOCAL n AS B392N
    LOCAL fw AS WSTRINGZ * %MAX_PATH

    fails = 0
    ver = VERSION$()
    IF ver <> "v0.2.180" THEN INCR fails

    #IF %DEF(%PB64_TARGET_64)
        target64 = 1
    #ELSE
        target64 = 0
    #ENDIF
    #IF %TARGET_64
        IF target64 <> 1 THEN INCR fails
    #ELSE
        IF target64 <> 0 THEN INCR fails
    #ENDIF

    IF %B392_OR <> 3 THEN INCR fails
    IF %B392_SYNC <> 1048576 THEN INCR fails

    ' BYTE-aligned: 1 + 260*2 + 4 = 525.
    IF SIZEOF(B392W) <> 525 THEN INCR fails
    IF SIZEOF(w.S) <> 520 THEN INCR fails
    IF SIZEOF(fw) <> 520 THEN INCR fails

    ' QWORD/QUAD aligned: BYTE 0, pad 7, LONG 8, tail 4 = 16.
    IF SIZEOF(B392Q) <> 16 THEN INCR fails
    IF SIZEOF(B392QUAD) <> 16 THEN INCR fails

    ' NATIVE: BYTE 0, LONG 4, QUAD 8 = 16 (not the QWORD layout 24).
    IF SIZEOF(B392N) <> 16 THEN INCR fails
    IF SIZEOF(n.C) <> 8 THEN INCR fails

    PRINT "batch392 WinAPI-layout checks done"
    OPEN "batch392_result.txt" FOR OUTPUT AS #9
    PRINT #9, "batch392 failures="; fails
    CLOSE #9
    FUNCTION = fails
END FUNCTION
