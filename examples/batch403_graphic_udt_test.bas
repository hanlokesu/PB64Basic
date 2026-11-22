#COMPILE EXE
#CONSOLE ON
#DIM ALL
#INCLUDE "Win32Api.inc"

' batch 403: pixel-level regression for GRAPHIC UDT targets, control attach,
' retained bitmap COPY, REDRAW and BOX fill.  This covers the Pong lvalue
' regression (GRAPHIC BITMAP NEW ... TO udt.member(index)) without the game.

TYPE SpriteSet
    Bmp AS LONG
    Letter(1 TO 5) AS LONG
END TYPE

GLOBAL Sprites AS SpriteSet

SUB ConPrint(BYVAL s AS STRING)
    PRINT #1, s
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL fails AS LONG
    LOCAL px AS LONG
    LOCAL hDlg AS LONG
    LOCAL i AS LONG

    OPEN "batch403_graphic_udt_out.txt" FOR OUTPUT AS #1

    ' UDT scalar member as bitmap target.
    GRAPHIC BITMAP NEW 40, 40 TO Sprites.Bmp
    IF Sprites.Bmp = 0 THEN
        ConPrint "FAIL: UDT scalar bitmap handle is zero"
        INCR fails
    ELSE
        ConPrint "OK: UDT scalar bitmap handle"
    END IF

    GRAPHIC ATTACH Sprites.Bmp, 0
    GRAPHIC CLEAR
    GRAPHIC BOX (5, 5)-(35, 35), 0, RGB(255, 0, 0), RGB(255, 0, 0)
    GRAPHIC GET PIXEL (20, 20) TO px
    IF px <> RGB(255, 0, 0) THEN
        ConPrint "FAIL: UDT bitmap pixel=" & STR$(px)
        INCR fails
    ELSE
        ConPrint "OK: UDT bitmap pixel"
    END IF

    ' UDT member array element as bitmap target (the Pong regression).
    FOR i = 1 TO 5
        GRAPHIC BITMAP NEW 40, 40 TO Sprites.Letter(i)
        IF Sprites.Letter(i) = 0 THEN
            ConPrint "FAIL: UDT member-array bitmap " & STR$(i) & " is zero"
            INCR fails
        END IF
    NEXT i
    IF fails = 0 THEN ConPrint "OK: UDT member-array bitmap targets"

    ' Control target, REDRAW, and COPY from a retained bitmap.
    DIALOG NEW 0, "GRAPHIC regression",, , 240, 160, %WS_CAPTION TO hDlg
    CONTROL ADD GRAPHIC, hDlg, 100, "", 0, 0, 200, 120
    GRAPHIC ATTACH hDlg, 100, REDRAW
    GRAPHIC CLEAR
    GRAPHIC BOX (5, 5)-(35, 35), 0, RGB(0, 0, 255), RGB(0, 0, 255)
    GRAPHIC COPY Sprites.Bmp, 0 TO (50, 10)
    GRAPHIC REDRAW
    GRAPHIC GET PIXEL (60, 20) TO px
    IF px <> RGB(255, 0, 0) THEN
        ConPrint "FAIL: copied pixel=" & STR$(px)
        INCR fails
    ELSE
        ConPrint "OK: control COPY pixel"
    END IF
    DestroyWindow hDlg

    IF fails = 0 THEN
        ConPrint "FAILURES:0"
    ELSE
        ConPrint "FAILURES:" & STR$(fails)
    END IF
    CLOSE #1
    FUNCTION = fails
END FUNCTION
