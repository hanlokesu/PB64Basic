#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 312 test
' Fix: PowerBASIC floating-point literal precision. A decimal literal with up
' to six written digits is rounded to Single precision (32-bit) and then
' widened, 7-16 digits use Double, and the explicit suffixes !/#/## force the
' precision. This matches the CHM "Constants and Literals" rules and makes the
' LPRINT double-probe output byte-identical to PBWin.
' Headless smoke: it writes verifiable lines with GetStdHandle/WriteFile and
' checks each expected value; there is no WAITKEY/MSGBOX tail.
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL h AS LONG, BYVAL t AS STRING)
    LOCAL w AS DWORD
    WriteFile h, BYVAL STRPTR(t), LEN(t), w, 0
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL h AS LONG
    LOCAL fails AS LONG
    LOCAL a AS DOUBLE
    LOCAL b AS DOUBLE
    LOCAL c AS DOUBLE
    LOCAL d AS DOUBLE
    LOCAL e AS DOUBLE
    h = GetStdHandle(-11)

    ' Six-digit literal 0.1 is rounded to Single, then widened to Double.
    a = 0.1
    ConPrint h, "single 0.1=" & TRIM$(STR$(a)) & $CRLF
    IF TRIM$(STR$(a)) <> ".100000001490116" THEN fails = fails + 1

    ' Seven-digit literal (trailing zero counts) stays Double and renders short.
    b = 0.1000000
    ConPrint h, "double 0.1000000=" & TRIM$(STR$(b)) & $CRLF
    IF TRIM$(STR$(b)) <> ".1" THEN fails = fails + 1

    ' Explicit # forces Double even on a six-digit literal.
    c = 0.1#
    ConPrint h, "forced 0.1#=" & TRIM$(STR$(c)) & $CRLF
    IF TRIM$(STR$(c)) <> ".1" THEN fails = fails + 1

    ' Explicit ! forces Single even on a seven-digit literal.
    d = 0.1000000!
    ConPrint h, "forced 0.1000000!=" & TRIM$(STR$(d)) & $CRLF
    IF TRIM$(STR$(d)) <> ".100000001490116" THEN fails = fails + 1

    ' Negative six-digit literal is rounded to Single.
    e = -123.456
    ConPrint h, "single -123.456=" & TRIM$(STR$(e)) & $CRLF
    IF TRIM$(STR$(e)) <> "-123.456001281738" THEN fails = fails + 1

    ' LPRINT$ returns an empty string when no device is attached.
    IF LPRINT$ <> "" THEN fails = fails + 1

    ConPrint h, "FAILS=" & TRIM$(fails) & $CRLF
    IF fails = 0 THEN ConPrint h, "batch312 test ok" & $CRLF
    FUNCTION = 0
END FUNCTION
