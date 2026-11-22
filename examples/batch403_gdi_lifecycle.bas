' batch 403: GRAPHIC control attach/destroy lifecycle.
' Creates two GRAPHIC controls, alternates their retained buffers, then
' destroys the dialog.  Repeats 50 times and checks that the process GDI
' handle count does not grow (one warm-up cycle absorbs one-time caches).
#COMPILE EXE
#CONSOLE ON
#DIM ALL
#INCLUDE "Win32Api.inc"

FUNCTION PBMAIN() AS LONG
    LOCAL i AS LONG
    LOCAL hDlg AS DWORD
    LOCAL gdiWarm AS LONG
    LOCAL gdiLive AS LONG
    LOCAL gdiAfter AS LONG
    LOCAL fnum AS LONG

    OPEN "batch403_gdi_lifecycle_result.txt" FOR OUTPUT AS #1
    fnum = 1

    ' Warm-up: create and destroy once so DDT/font caches are already allocated.
    DIALOG NEW 0, "GDI lifecycle",, , 220, 140, %WS_CAPTION TO hDlg
    CONTROL ADD GRAPHIC, hDlg, 100, "", 0, 0, 100, 100
    CONTROL ADD GRAPHIC, hDlg, 101, "", 105, 0, 100, 100
    GRAPHIC ATTACH hDlg, 100
    GRAPHIC CLEAR
    GRAPHIC BOX (5, 5)-(95, 95), 0, RGB(255, 0, 0), RGB(0, 0, 255)
    GRAPHIC ATTACH hDlg, 101
    GRAPHIC CLEAR
    GRAPHIC BOX (5, 5)-(95, 95), 0, RGB(0, 255, 0), RGB(255, 255, 0)
    GRAPHIC ATTACH hDlg, 100
    GRAPHIC PRINT "OK"
    gdiLive = GetGuiResources(GetCurrentProcess(), 0)
    DestroyWindow hDlg

    gdiWarm = GetGuiResources(GetCurrentProcess(), 0)

    FOR i = 1 TO 50
        DIALOG NEW 0, "GDI lifecycle",, , 220, 140, %WS_CAPTION TO hDlg
        CONTROL ADD GRAPHIC, hDlg, 100, "", 0, 0, 100, 100
        CONTROL ADD GRAPHIC, hDlg, 101, "", 105, 0, 100, 100
        GRAPHIC ATTACH hDlg, 100
        GRAPHIC CLEAR
        GRAPHIC BOX (5, 5)-(95, 95), 0, RGB(255, 0, 0), RGB(0, 0, 255)
        GRAPHIC ATTACH hDlg, 101
        GRAPHIC CLEAR
        GRAPHIC BOX (5, 5)-(95, 95), 0, RGB(0, 255, 0), RGB(255, 255, 0)
        GRAPHIC ATTACH hDlg, 100
        GRAPHIC PRINT "OK"
        DestroyWindow hDlg
    NEXT i

    gdiAfter = GetGuiResources(GetCurrentProcess(), 0)
    PRINT #fnum, "GDI_LIVE="; gdiLive; " GDI_WARM="; gdiWarm; " GDI_AFTER="; gdiAfter; " DELTA="; gdiAfter - gdiWarm
    IF gdiAfter - gdiWarm <= 2 THEN
        PRINT #fnum, "GDI_LIFECYCLE_PASS"
    ELSE
        PRINT #fnum, "GDI_LIFECYCLE_FAIL"
    END IF
    CLOSE #fnum
END FUNCTION
