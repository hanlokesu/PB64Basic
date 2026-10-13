#COMPILE EXE

' --- PBWin10 stub branch: this sample exercises PowerBasilisk-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.


' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB
DECLARE FUNCTION ReleaseDC LIB "USER32.DLL" ALIAS "ReleaseDC" (BYVAL hWnd AS DWORD, BYVAL hDC AS DWORD) AS LONG
DECLARE FUNCTION IsWindowVisible LIB "USER32.DLL" ALIAS "IsWindowVisible" (BYVAL hWnd AS DWORD) AS LONG
DECLARE FUNCTION GetPixel LIB "GDI32.DLL" ALIAS "GetPixel" (BYVAL hDC AS DWORD, BYVAL X AS LONG, BYVAL Y AS LONG) AS DWORD
DECLARE FUNCTION GetDC LIB "USER32.DLL" ALIAS "GetDC" (BYVAL hWnd AS DWORD) AS DWORD

DECLARE FUNCTION IsWindow LIB "USER32.DLL" ALIAS "IsWindow" (BYVAL hWnd AS DWORD) AS LONG
SUB ConWaitKey()
    LOCAL h AS LONG
    LOCAL c AS STRING * 1
    LOCAL n AS DWORD
    h = GetStdHandle(-10)
    IF h <> 0 THEN
        ReadFile h, c, 1, n, 0
    END IF
END SUB

'=====================================================================
' batch181_test.bas - the six GRAPHIC statements of batch 181
'---------------------------------------------------------------------
' Purpose:   Exercise every statement added in batch 181:
'
'   GRAPHIC REDRAW                          (flush a buffered target)
'   GRAPHIC SET FOCUS                       (bring the window forward, focus)
'   GRAPHIC SET LOC x&, y&                  (move the graphic window)
'   GRAPHIC SET CLIENT nWide&, nHigh&       (resize the client area)
'   GRAPHIC SET OVERLAP [NumrExpr&]         (RECT edges become inclusive)
'   GRAPHIC GET OVERLAP To OverlapVar&      (read that flag back)
'
' Official sources (C:\PBWin10\bin\PBWin_extracted\html):
'   GRAPHIC_REDRAW_statement.htm            "Update buffered graphical statements,
'                                            drawing them to the selected graphic
'                                            target ... only needed when GRAPHIC
'                                            ATTACH with the REDRAW option have
'                                            been chosen."
'   GRAPHIC_SET_FOCUS_statement.htm         "Bring the selected graphic window
'                                            to the foreground and direct focus."
'   GRAPHIC_SET_LOC_statement.htm           pixels, relative to the upper left
'                                            corner of the screen
'   GRAPHIC_SET_CLIENT_statement.htm        client-area size of a graphic control
'                                            or graphic window
'   GRAPHIC_SET_OVERLAP_statement.htm       non-zero enables, zero disables,
'                                            missing means enable; BOX (0,0)-(50,50)
'                                            then reaches 50,50 instead of 49,49
'   GRAPHIC_GET_OVERLAP_statement.htm       true (non-zero) / false (zero)
'   GRAPHIC_GET_LOC_statement.htm           used here to verify SET LOC
'
' Demonstrates:
'   * a graphic window's display is buffered, so every pixel read below is taken
'     after GRAPHIC REDRAW has pushed the buffer out - which is exactly the job
'     that statement has, and why the first block both draws and flushes.
'   * the overlap flag starts disabled - Windows' own RECT convention, where the
'     right and bottom edges of a rectangle are exclusive - and every documented
'     way of setting it (explicit zero, explicit one, missing operand).
'   * the flag really changes what reaches the pixels: the same filled box paints
'     its bottom-right corner only when overlap mode is on.  The flag is one
'     runtime-wide switch, not one per target, which is stated here rather than
'     left to be discovered.
'   * SET CLIENT round-trips through GRAPHIC GET CLIENT (which reports the
'     drawing buffer's dimensions), and drawing still works at the new size.
'   * SET LOC round-trips through GRAPHIC GET LOC, which batch 181 also had to
'     fix: it used to answer 0,0 unconditionally, which is only correct for the
'     "not a Graphic Window" case the help page describes.
'   * SET FOCUS runs without disturbing the window (the foreground state itself
'     cannot be asserted from an unattended run - Windows only grants focus to a
'     process that is already in the foreground).
'
' What this sample deliberately does NOT cover (and why):
'   * the overlap rule for LINE / POLYLINE: their endpoints are already exclusive
'     in GDI terms (LineTo does not paint its final point), so the flag is applied
'     to the RECT-based statements - BOX and ELLIPSE - which is where the help
'     page's worked example lives.  Recorded as a known limit, not a silent one.
'   * dialog-unit sizing of SET CLIENT: a DDT graphic control created from dialog
'     units would scale nWide&/nHigh& by the dialog base units; this runtime
'     treats them as pixels.
'   * the greyed-out corner of a rounded box: the optional corner operand belongs
'     to the BOX statement, not to this batch.
'
' Expected output (run mode): a series of "ok" lines and
'   "=== FAILURES:0 ===", exit code 0.
' Complexity note: O(1) - a fixed list of draws and pixel reads.
'=====================================================================
#COMPILER PBWIN 10

%RED     = 255      ' COLORREF 0x000000FF
%GREEN   = 65280    ' COLORREF 0x0000FF00
%BLUE    = 16711680 ' COLORREF 0x00FF0000

FUNCTION PBMAIN () AS LONG
    LOCAL hGr    AS LONG
    LOCAL iswA   AS LONG
    LOCAL iswH   AS LONG
    LOCAL isvA   AS LONG
    LOCAL isvH   AS LONG
    LOCAL dcA    AS LONG
    LOCAL dcH    AS LONG
    LOCAL rdA    AS LONG
    LOCAL rdH    AS LONG
    LOCAL gpA    AS LONG
    LOCAL gpH    AS LONG
    LOCAL hdc    AS LONG
    LOCAL st     AS LONG
    LOCAL px     AS LONG
    LOCAL ov     AS LONG
    LOCAL lx     AS LONG
    LOCAL ly     AS LONG
    LOCAL cw     AS LONG
    LOCAL ch     AS LONG
    LOCAL fail   AS LONG

    ConPrint "batch 181 - GRAPHIC REDRAW / SET FOCUS / SET LOC / SET CLIENT /"
    ConPrint "            SET OVERLAP / GET OVERLAP"
    ConPrint "-------------------------------------------------------------"

    IMPORT ADDR "IsWindow",        "USER32.DLL" TO iswA, iswH
    IMPORT ADDR "IsWindowVisible", "USER32.DLL" TO isvA, isvH
    IMPORT ADDR "GetDC",           "USER32.DLL" TO dcA, dcH
    IMPORT ADDR "ReleaseDC",       "USER32.DLL" TO rdA, rdH
    IMPORT ADDR "GetPixel",        "GDI32.DLL"  TO gpA, gpH
    IF iswA = 0 OR isvA = 0 OR dcA = 0 OR rdA = 0 OR gpA = 0 THEN
        fail = fail + 1
        ConPrint "FAIL IMPORT ADDR (USER32/GDI32 helpers)"
    END IF

    ' ------------------------------------------------------------------
    ' The target: a live graphic window, which NEW selects for drawing.
    ' ------------------------------------------------------------------
    hGr = 0
    GRAPHIC WINDOW NEW "batch 181 - overlap / client / loc", 60, 60, 340, 260 TO hGr
    IF hGr <> 0 THEN
        ConPrint "ok   GRAPHIC WINDOW NEW returned a handle"
    ELSE
        fail = fail + 1
        ConPrint "FAIL GRAPHIC WINDOW NEW handle"
    END IF

    st = 0
    CALL DWORD iswA USING IsWindow(hGr) TO st
    IF st <> 0 THEN
        ConPrint "ok   the drawing target is a live window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IsWindow(hGr)=" & STR$(st)
    END IF

    ' ------------------------------------------------------------------
    ' Overlap is off out of the box - the Windows RECT convention.
    ' ------------------------------------------------------------------
    ov = -1
    GRAPHIC GET OVERLAP TO ov
    IF ov = 0 THEN
        ConPrint "ok   GET OVERLAP answers 0 before anything is set"
    ELSE
        fail = fail + 1
        ConPrint "FAIL default overlap =" & STR$(ov)
    END IF

    ' ------------------------------------------------------------------
    ' REDRAW, and the exclusive corner it implies while overlap is off.
    ' ------------------------------------------------------------------
    GRAPHIC BOX (10,10)-(60,60), %RED, %RED, 1
    GRAPHIC REDRAW
    hdc = 0
    CALL DWORD dcA USING GetDC(hGr) TO hdc
    px = -1
    CALL DWORD gpA USING GetPixel(hdc, 59, 59) TO px
    IF px = %RED THEN
        ConPrint "ok   GRAPHIC REDRAW flushed the buffer (interior pixel 59,59)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL pixel after REDRAW=" & STR$(px)
    END IF

    px = -1
    CALL DWORD gpA USING GetPixel(hdc, 60, 60) TO px
    IF px <> %RED THEN
        ConPrint "ok   overlap off: the exclusive corner (60,60) is untouched"
    ELSE
        fail = fail + 1
        ConPrint "FAIL exclusive corner was painted with overlap off"
    END IF

    ' ------------------------------------------------------------------
    ' SET OVERLAP 1 - and the corner pixel now belongs to the rectangle.
    ' ------------------------------------------------------------------
    GRAPHIC SET OVERLAP 1
    ov = 0
    GRAPHIC GET OVERLAP TO ov
    IF ov <> 0 THEN
        ConPrint "ok   SET OVERLAP 1 is reported back by GET OVERLAP"
    ELSE
        fail = fail + 1
        ConPrint "FAIL GET OVERLAP after SET OVERLAP 1 =" & STR$(ov)
    END IF

    GRAPHIC BOX (100,100)-(150,150), %BLUE, %BLUE, 1
    GRAPHIC REDRAW
    px = -1
    CALL DWORD gpA USING GetPixel(hdc, 150, 150) TO px
    IF px = %BLUE THEN
        ConPrint "ok   overlap on: BOX (100,100)-(150,150) reaches (150,150)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL inclusive corner pixel=" & STR$(px)
    END IF

    ' The same flag drives ELLIPSE, the other RECT-based statement.
    GRAPHIC ELLIPSE (200,10)-(250,60), %GREEN, %GREEN, 1
    GRAPHIC REDRAW
    px = -1
    CALL DWORD gpA USING GetPixel(hdc, 250, 35) TO px
    IF px = %GREEN THEN
        ConPrint "ok   overlap on: the ELLIPSE edge reaches its right extreme"
    ELSE
        fail = fail + 1
        ConPrint "FAIL ellipse edge pixel=" & STR$(px)
    END IF

    ' ------------------------------------------------------------------
    ' Back to the default, then the operand-less form.
    ' ------------------------------------------------------------------
    GRAPHIC SET OVERLAP 0
    ov = -1
    GRAPHIC GET OVERLAP TO ov
    IF ov = 0 THEN
        ConPrint "ok   SET OVERLAP 0 turns the mode back off"
    ELSE
        fail = fail + 1
        ConPrint "FAIL GET OVERLAP after SET OVERLAP 0 =" & STR$(ov)
    END IF

    GRAPHIC SET OVERLAP
    ov = 0
    GRAPHIC GET OVERLAP TO ov
    IF ov <> 0 THEN
        ConPrint "ok   a missing operand enables overlap mode"
    ELSE
        fail = fail + 1
        ConPrint "FAIL missing operand left overlap off"
    END IF
    GRAPHIC SET OVERLAP 0

    ' ------------------------------------------------------------------
    ' SET CLIENT - round-trips through GET CLIENT, and drawing still works.
    ' ------------------------------------------------------------------
    GRAPHIC SET CLIENT 200, 120
    cw = 0
    ch = 0
    GRAPHIC GET CLIENT TO cw, ch
    IF cw = 200 AND ch = 120 THEN
        ConPrint "ok   SET CLIENT 200,120 is reported back by GET CLIENT"
    ELSE
        fail = fail + 1
        ConPrint "FAIL GET CLIENT after SET CLIENT =" & STR$(cw) & "," & STR$(ch)
    END IF

    GRAPHIC CLEAR %GREEN
    GRAPHIC REDRAW
    px = -1
    CALL DWORD gpA USING GetPixel(hdc, 100, 60) TO px
    IF px = %GREEN THEN
        ConPrint "ok   the resized buffer takes new drawing"
    ELSE
        fail = fail + 1
        ConPrint "FAIL pixel in the resized buffer=" & STR$(px)
    END IF

    ' ------------------------------------------------------------------
    ' SET LOC / GET LOC - the window moves to the screen pixels given.
    ' ------------------------------------------------------------------
    GRAPHIC SET LOC 260, 180
    lx = -1
    ly = -1
    GRAPHIC GET LOC TO lx, ly
    IF lx = 260 AND ly = 180 THEN
        ConPrint "ok   SET LOC 260,180 is reported back by GET LOC"
    ELSE
        fail = fail + 1
        ConPrint "FAIL GET LOC after SET LOC =" & STR$(lx) & "," & STR$(ly)
    END IF

    ' ------------------------------------------------------------------
    ' SET FOCUS - no crash, and the window is untouched by it.
    ' ------------------------------------------------------------------
    GRAPHIC SET FOCUS
    st = 0
    CALL DWORD iswA USING IsWindow(hGr) TO st
    IF st <> 0 THEN
        ConPrint "ok   SET FOCUS left the window alive"
    ELSE
        fail = fail + 1
        ConPrint "FAIL window gone after SET FOCUS"
    END IF

    lx = 0
    ly = 0
    GRAPHIC GET LOC TO lx, ly
    IF lx = 260 AND ly = 180 THEN
        ConPrint "ok   SET FOCUS did not move the window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL position changed to" & STR$(lx) & "," & STR$(ly)
    END IF

    st = 0
    CALL DWORD isvA USING IsWindowVisible(hGr) TO st
    IF st <> 0 THEN
        ConPrint "ok   the window is still visible"
    ELSE
        fail = fail + 1
        ConPrint "FAIL window no longer visible"
    END IF

    st = 0
    CALL DWORD rdA USING ReleaseDC(hGr, hdc) TO st

    ' ------------------------------------------------------------------
    ' END the window: the statements above must go quiet, not crash.
    ' ------------------------------------------------------------------
    GRAPHIC WINDOW END hGr
    st = 0
    CALL DWORD iswA USING IsWindow(hGr) TO st
    IF st = 0 THEN
        ConPrint "ok   GRAPHIC WINDOW END destroyed the window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL window survived END"
    END IF

    ' With no target at all, REDRAW / SET FOCUS / GET LOC are no-ops, and GET LOC
    ' answers the documented 0,0.
    GRAPHIC REDRAW
    GRAPHIC SET FOCUS
    lx = -1
    ly = -1
    GRAPHIC GET LOC TO lx, ly
    IF lx = 0 AND ly = 0 THEN
        ConPrint "ok   with no graphic window, GET LOC answers 0,0"
    ELSE
        fail = fail + 1
        ConPrint "FAIL GET LOC with no window =" & STR$(lx) & "," & STR$(ly)
    END IF

    ConPrint "-------------------------------------------------------------"
    ConPrint "=== FAILURES:" & STR$(fail) & " ==="
    FUNCTION = fail
END FUNCTION



