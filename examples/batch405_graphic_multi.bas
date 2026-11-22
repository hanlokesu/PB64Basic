' batch 405: multi-control / multi-window GRAPHIC lifecycle regression.
' Each GRAPHIC control owns an independent buffer. Destroying one control
' must not clear or alias another control's buffer.
#COMPILE EXE
#CONSOLE ON
#DIM ALL
#INCLUDE "Win32Api.inc"

GLOBAL gFail AS LONG
GLOBAL gOut AS LONG

SUB CheckPixel(BYVAL hDlg AS DWORD, BYVAL id AS LONG, BYVAL x AS LONG, BYVAL y AS LONG, BYVAL want AS LONG, msg AS STRING)
    LOCAL hCap AS DWORD
    LOCAL px AS LONG
    GRAPHIC ATTACH hDlg, id
    GRAPHIC BITMAP CAPTURE TO hCap
    IF hCap = 0 THEN
        INCR gFail
        PRINT #gOut, "FAIL "; msg; " capture=0"
        EXIT SUB
    END IF
    GRAPHIC ATTACH hCap, 0
    GRAPHIC GET PIXEL (x, y) TO px
    GRAPHIC BITMAP END hCap
    IF px <> want THEN
        INCR gFail
        PRINT #gOut, "FAIL "; msg; " pixel("; x; ","; y; ") got="; px; " want="; want
    ELSE
        PRINT #gOut, "PASS "; msg
    END IF
END SUB

FUNCTION PBMAIN () AS LONG
    LOCAL hDlg AS DWORD
    LOCAL hwndB AS DWORD
    LOCAL hwndDead AS DWORD
    LOCAL hBmp AS DWORD
    LOCAL px AS LONG
    gFail = 0
    OPEN "batch405_multi_result.txt" FOR OUTPUT AS #1
    gOut = 1

    DIALOG NEW 0, "GRAPHIC multi-control probe", -2000, 0, 360, 140, %WS_CAPTION TO hDlg
    CONTROL ADD GRAPHIC, hDlg, 100, "", 0, 0, 100, 100
    CONTROL ADD GRAPHIC, hDlg, 101, "", 110, 0, 100, 100
    CONTROL ADD GRAPHIC, hDlg, 102, "", 220, 0, 100, 100
    DIALOG SHOW MODELESS hDlg
    DIALOG DOEVENTS

    GRAPHIC ATTACH hDlg, 100
    GRAPHIC CLEAR RGB(255, 0, 0)
    CheckPixel hDlg, 100, 50, 50, RGB(255, 0, 0), "control 100 initial red"

    GRAPHIC ATTACH hDlg, 101
    GRAPHIC CLEAR RGB(0, 255, 0)
    CheckPixel hDlg, 101, 50, 50, RGB(0, 255, 0), "control 101 initial green"

    GRAPHIC ATTACH hDlg, 102
    GRAPHIC CLEAR RGB(0, 0, 255)
    CheckPixel hDlg, 102, 50, 50, RGB(0, 0, 255), "control 102 initial blue"

    ' Re-select control 100; it must still contain red, not the last blue buffer.
    GRAPHIC ATTACH hDlg, 100
    CheckPixel hDlg, 100, 50, 50, RGB(255, 0, 0), "control 100 survives 101/102 draws"

    ' GRAPHIC COPY from a real child-window handle with id 0 uses the
    ' control's backing buffer, not a GetDC snapshot.
    GRAPHIC BITMAP NEW 40, 40 TO hBmp
    GRAPHIC ATTACH hBmp, 0
    GRAPHIC CLEAR RGB(0, 0, 0)
    hwndB = GetDlgItem(hDlg, 102)
    GRAPHIC COPY hwndB, 0, (0, 0)-(39, 39) TO (0, 0)
    GRAPHIC GET PIXEL (10, 10) TO px
    GRAPHIC BITMAP END hBmp
    IF px <> RGB(0, 0, 255) THEN
        INCR gFail
        PRINT #gOut, "FAIL copy live hwnd,0 got="; px
    ELSE
        PRINT #gOut, "PASS copy live hwnd,0"
    END IF

    ' Destroy the middle control and keep drawing on both neighbours.
    hwndB = GetDlgItem(hDlg, 101)
    hwndDead = hwndB
    DestroyWindow hwndB
    DIALOG DOEVENTS

    ' A copy from a destroyed window must fail without damaging the target.
    GRAPHIC BITMAP NEW 40, 40 TO hBmp
    GRAPHIC ATTACH hBmp, 0
    GRAPHIC CLEAR RGB(255, 0, 255)
    GRAPHIC COPY hwndDead, 0, (0, 0)-(39, 39) TO (0, 0)
    GRAPHIC GET PIXEL (10, 10) TO px
    GRAPHIC BITMAP END hBmp
    IF px <> RGB(255, 0, 255) THEN
        INCR gFail
        PRINT #gOut, "FAIL copy dead hwnd damaged target: "; px
    ELSE
        PRINT #gOut, "PASS copy dead hwnd leaves target intact"
    END IF

    GRAPHIC ATTACH hDlg, 100
    GRAPHIC CLEAR RGB(255, 255, 0)
    GRAPHIC BOX (10, 10)-(90, 90), 0, RGB(0, 0, 0), RGB(255, 255, 0)
    CheckPixel hDlg, 100, 50, 50, RGB(255, 255, 0), "control 100 draws after 101 destroyed"

    GRAPHIC ATTACH hDlg, 102
    GRAPHIC CLEAR RGB(0, 255, 255)
    CheckPixel hDlg, 102, 50, 50, RGB(0, 255, 255), "control 102 draws after 101 destroyed"

    GRAPHIC ATTACH hDlg, 100
    CheckPixel hDlg, 100, 50, 50, RGB(255, 255, 0), "control 100 not overwritten by control 102"

    ' The dialog background is a separate target id 0.
    GRAPHIC ATTACH hDlg, 0
    GRAPHIC CLEAR RGB(255, 255, 255)
    CheckPixel hDlg, 0, 350, 10, RGB(255, 255, 255), "dialog id 0 background"

    GRAPHIC ATTACH hDlg, 100
    GRAPHIC REDRAW
    GRAPHIC ATTACH hDlg, 102
    GRAPHIC REDRAW

    DestroyWindow hDlg
    IF gFail = 0 THEN
        PRINT #gOut, "batch405_multi: ALL PASS"
    ELSE
        PRINT #gOut, "batch405_multi: FAILURES="; gFail
    END IF
    CLOSE #gOut
    FUNCTION = gFail
END FUNCTION
