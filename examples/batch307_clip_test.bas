#COMPILE EXE
#DIM ALL
' _batch307_clip_test.bas - CLIPBOARD BITMAP true roundtrip (legacy #10)
' Build a real 32x16 bitmap (DDB via CreateCompatibleBitmap - works for both
' PBWin10's GRAPHIC-flavoured CLIPBOARD SET BITMAP and PB64Basic), SET it, GET
' it back, verify size via GetObject.  Dual-run: PBWin10 and PB64Basic must
' print the same line to batch307_clip_out.txt.

TYPE BITMAPT
    bmType AS LONG
    bmWidth AS LONG
    bmHeight AS LONG
    bmWidthBytes AS LONG
    bmPlanes AS WORD
    bmBitsPixel AS WORD
    bmBits AS LONG
    bmBitsHi AS LONG
    bmPad AS LONG
END TYPE

DECLARE FUNCTION CreateCompatibleBitmap LIB "GDI32.DLL" ALIAS "CreateCompatibleBitmap" (BYVAL hdc AS LONG, BYVAL nWidth AS LONG, BYVAL nHeight AS LONG) AS LONG
DECLARE FUNCTION GetDC LIB "USER32.DLL" ALIAS "GetDC" (BYVAL hwnd AS LONG) AS LONG
DECLARE FUNCTION ReleaseDC LIB "USER32.DLL" ALIAS "ReleaseDC" (BYVAL hwnd AS LONG, BYVAL hdc AS LONG) AS LONG
DECLARE FUNCTION GetObject LIB "GDI32.DLL" ALIAS "GetObjectA" (BYVAL hgdiobj AS LONG, BYVAL cbBuffer AS DWORD, BYVAL lpvObject AS LONG) AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL hdc AS LONG
    LOCAL hbm AS LONG
    LOCAL hbm2 AS LONG
    LOCAL cr0 AS LONG
    LOCAL cr1 AS LONG
    LOCAL cr2 AS LONG
    LOCAL bm AS BITMAPT
    LOCAL grc AS LONG
    LOCAL ok AS LONG

    hdc = GetDC(0)
    hbm = CreateCompatibleBitmap(hdc, 32, 16)
    ReleaseDC 0, hdc
    IF hbm = 0 THEN
        OPEN "batch307_clip_out.txt" FOR OUTPUT AS #1
        PRINT #1, "clip: bitmap create failed"
        CLOSE #1
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    CLIPBOARD RESET, cr0
    CLIPBOARD SET BITMAP hbm, cr1
    CLIPBOARD GET BITMAP TO hbm2, cr2

    grc = GetObject(hbm2, SIZEOF(bm), VARPTR(bm))
    IF grc AND bm.bmWidth = 32 AND bm.bmHeight = 16 THEN
        ok = -1
    ELSE
        ok = 0
    END IF

    OPEN "batch307_clip_out.txt" FOR OUTPUT AS #1
    PRINT #1, "clip: cr0=" & STR$(cr0) & " cr1=" & STR$(cr1) & " cr2=" & STR$(cr2) & _
               " w=" & STR$(bm.bmWidth) & " h=" & STR$(bm.bmHeight) & " ok=" & STR$(ok)
    CLOSE #1
    FUNCTION = 0
END FUNCTION
