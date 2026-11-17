' batch378_input - interactive keyboard/mouse probe.
' Run in a real console, then type the requested lines and click once.
' CON.INPUT / CON.LINE.INPUT read CONIN$, not redirected stdin; this probe
' therefore must be driven by real console input, not a pipe.
#COMPILE EXE
#CONSOLE ON
#DIM ALL

FUNCTION HexByte(v AS LONG) AS STRING
    FUNCTION = HEX$(v, 2)
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL n AS LONG
    LOCAL s AS STRING
    LOCAL line AS STRING
    LOCAL k AS STRING
    LOCAL a AS LONG
    LOCAL i AS LONG

    OPEN "batch378_input.txt" FOR OUTPUT AS #1

    CON.STDERR "input: type 123,alpha then Enter" + CHR$(13, 10)
    CON.INPUT "num,word", n, s
    PRINT #1, "n=" + STR$(n) + " s=[" + s + "]"

    CON.INPUT.FLUSH
    SLEEP 100
    CON.STDERR "line: type hello world then Enter" + CHR$(13, 10)
    CON.LINE.INPUT "line", line
    PRINT #1, "line=[" + line + "]"

    CON.INPUT.FLUSH
    SLEEP 100
    CON.STDERR "waitstat: press Z" + CHR$(13, 10)
    CON.WAITSTAT
    k = CON.INKEY$
    PRINT #1, "key len=" + STR$(LEN(k)) + " hex=" + HexByte(ASC(k))

    CON.STDERR "mouse: click once in this window" + CHR$(13, 10)
    CON.MOUSE 15, 31
    CON.MOUSE.ON
    CON.WAITSTAT
    a = CON.MOUSE.STAT
    ' A click first queues a MOUSE_MOVED record, then the press/release pair.
    ' WAITKEY$ skips the move (button mask 0) and returns the first accepted
    ' button event as a 4-byte string.
    k = CON.WAITKEY$("", 2000)
    PRINT #1, "mouse stat=" + STR$(a) + " keylen=" + STR$(LEN(k))
    IF LEN(k) >= 4 THEN
        PRINT #1, "mouse key=" + HexByte(ASC(k, 1)) + HexByte(ASC(k, 2)) + HexByte(ASC(k, 3)) + HexByte(ASC(k, 4))
    END IF
    PRINT #1, "mouse row=" + STR$(CON.MOUSE.ROW) + " col=" + STR$(CON.MOUSE.COL)
    PRINT #1, "mouse buttons=" + STR$(CON.MOUSE.BUTTONS)
    CON.MOUSE.OFF

    CLOSE #1
    CON.STDERR "input: done" + CHR$(13, 10)
END FUNCTION
