' batch 403: GRAPHIC SET PIXEL and hollow BOX/ELLIPSE regression.
' The BMP is parsed externally: (10,10) must be red, the ellipse border
' blue and its interior white because an omitted fill is hollow.
#COMPILE EXE
#CONSOLE ON
#DIM ALL
#INCLUDE "Win32Api.inc"

FUNCTION PBMAIN () AS LONG
    LOCAL hBmp AS DWORD

    GRAPHIC BITMAP NEW 40, 40 TO hBmp
    GRAPHIC ATTACH hBmp, 0
    GRAPHIC CLEAR RGB(255, 255, 255)
    GRAPHIC SET PIXEL (10, 10), RGB(255, 0, 0)
    GRAPHIC ELLIPSE (15, 15)-(25, 25), RGB(0, 0, 255)
    GRAPHIC SAVE "batch403_gdi_semantics.bmp"
    GRAPHIC BITMAP END hBmp

    PRINT "FAILURES: 0"
END FUNCTION
