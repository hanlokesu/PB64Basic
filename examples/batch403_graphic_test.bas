' batch 403: GRAPHIC attach / bitmap / copy / capture / box regression.
' Pixel assertions are made on BMPs written by GRAPHIC BITMAP CAPTURE + SAVE,
' so the probe does not depend on 32-bit HDC declarations in Win32Api.inc.
' x64 and i686 must both print FAILURES: 0 and produce all capture BMPs.
#COMPILE EXE
#CONSOLE ON
#DIM ALL
#INCLUDE "Win32Api.inc"

TYPE BmpBox
    h AS DWORD
END TYPE

GLOBAL gFail AS LONG
GLOBAL gOut  AS LONG

SUB Check(BYVAL cond AS LONG, msg AS STRING)
    IF cond = 0 THEN
        INCR gFail
        PRINT #gOut, "FAIL "; msg
    ELSE
        PRINT #gOut, "PASS "; msg
    END IF
END SUB

SUB Snap(BYVAL hDlg AS DWORD, BYVAL id AS LONG, fname AS STRING)
    LOCAL hCap AS DWORD
    GRAPHIC ATTACH hDlg, id
    GRAPHIC BITMAP CAPTURE TO hCap
    Check hCap <> 0, "capture " + fname
    IF hCap <> 0 THEN
        GRAPHIC ATTACH hCap, 0
        GRAPHIC SAVE fname
        GRAPHIC BITMAP END hCap
    END IF
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL hDlg   AS DWORD
    LOCAL hBmp   AS DWORD
    LOCAL hwndB  AS DWORD
    LOCAL u      AS BmpBox

    OPEN "batch403_graphic_test_result.txt" FOR OUTPUT AS #1
    gOut = 1
    gFail = 0

    DIALOG NEW 0, "GRAPHIC probe", -2000, 0, 240, 180, %WS_CAPTION TO hDlg
    CONTROL ADD GRAPHIC, hDlg, 100, "", 0, 0, 110, 110
    CONTROL ADD GRAPHIC, hDlg, 101, "", 120, 0, 110, 110
    DIALOG SHOW MODELESS hDlg
    DIALOG DOEVENTS

    GRAPHIC BITMAP NEW 40, 40 TO hBmp
    Check hBmp <> 0, "bitmap new local handle"
    GRAPHIC ATTACH hBmp, 0
    GRAPHIC CLEAR RGB(255, 0, 0)

    GRAPHIC ATTACH hDlg, 100
    GRAPHIC CLEAR RGB(0, 0, 255)
    GRAPHIC COPY hBmp, 0, (0, 0)-(39, 39) TO (10, 10)
    GRAPHIC REDRAW
    Snap hDlg, 100, "batch403_cap_copy1.bmp"

    GRAPHIC BITMAP NEW 40, 40 TO u.h
    Check u.h <> 0, "bitmap new UDT field handle"
    GRAPHIC ATTACH u.h, 0
    GRAPHIC CLEAR RGB(0, 255, 0)
    GRAPHIC ATTACH hDlg, 100
    GRAPHIC COPY u.h, 0, (0, 0)-(39, 39) TO (10, 60)
    GRAPHIC REDRAW
    Snap hDlg, 100, "batch403_cap_copy2.bmp"

    GRAPHIC ATTACH hDlg, 101
    GRAPHIC CLEAR RGB(255, 0, 0)
    GRAPHIC REDRAW
    hwndB = GetDlgItem(hDlg, 101)
    GRAPHIC ATTACH hDlg, 100
    GRAPHIC COPY hwndB, 0, (0, 0)-(39, 39) TO (60, 10)
    GRAPHIC REDRAW
    Snap hDlg, 100, "batch403_cap_copy3.bmp"

    DestroyWindow hwndB
    GRAPHIC ATTACH hDlg, 100
    GRAPHIC BOX (60, 60)-(99, 99), 0, RGB(0, 0, 255), RGB(0, 0, 255)
    GRAPHIC REDRAW
    Snap hDlg, 100, "batch403_cap_draw.bmp"
    GRAPHIC COPY hwndB, 0, (0, 0)-(39, 39) TO (60, 60)
    GRAPHIC REDRAW
    Snap hDlg, 100, "batch403_cap_reject.bmp"

    GRAPHIC ATTACH hDlg, 0
    GRAPHIC BOX (120, 120)-(200, 170), 0, RGB(255, 255, 0), RGB(255, 255, 0)
    GRAPHIC REDRAW
    Snap hDlg, 0, "batch403_cap_dialog.bmp"

    GRAPHIC BITMAP NEW 100, 100 TO hBmp
    GRAPHIC ATTACH hBmp, 0
    GRAPHIC CLEAR RGB(0, 0, 0)
    GRAPHIC BOX (0, 0)-(99, 99), 0, RGB(255, 255, 0), RGB(255, 255, 0), 0
    GRAPHIC BOX (0, 0)-(99, 99), 0, RGB(255, 0, 255), RGB(255, 0, 255), 100
    GRAPHIC BOX (99, 99)-(0, 0), 0, RGB(0, 255, 255), RGB(0, 255, 255), -1
    GRAPHIC SAVE "batch403_box.bmp"
    GRAPHIC BITMAP END hBmp

    DestroyWindow hDlg
    PRINT #gOut, "FAILURES: "; gFail
    CLOSE #gOut
END FUNCTION
