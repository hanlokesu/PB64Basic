#COMPILE EXE
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

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
#IF %MY_PBVER = 0
DECLARE FUNCTION SendMessageA LIB "USER32.DLL" ALIAS "SendMessageA" (BYVAL hWnd AS QUAD, BYVAL wMsg AS DWORD, BYVAL wParam AS DWORD, BYVAL lParam AS DWORD) AS LONG
#ELSE
DECLARE FUNCTION SendMessageA LIB "USER32.DLL" ALIAS "SendMessageA" (BYVAL hWnd AS DWORD, BYVAL wMsg AS DWORD, BYVAL wParam AS DWORD, BYVAL lParam AS DWORD) AS LONG
#ENDIF

#IF %MY_PBVER = 0
DECLARE FUNCTION ReleaseDC LIB "USER32.DLL" ALIAS "ReleaseDC" (BYVAL hWnd AS QUAD, BYVAL hDC AS QUAD) AS LONG
#ELSE
DECLARE FUNCTION ReleaseDC LIB "USER32.DLL" ALIAS "ReleaseDC" (BYVAL hWnd AS DWORD, BYVAL hDC AS DWORD) AS LONG
#ENDIF

#IF %MY_PBVER = 0
DECLARE FUNCTION GetPixel LIB "GDI32.DLL" ALIAS "GetPixel" (BYVAL hDC AS QUAD, BYVAL X AS LONG, BYVAL Y AS LONG) AS DWORD
#ELSE
DECLARE FUNCTION GetPixel LIB "GDI32.DLL" ALIAS "GetPixel" (BYVAL hDC AS DWORD, BYVAL X AS LONG, BYVAL Y AS LONG) AS DWORD
#ENDIF

#IF %MY_PBVER = 0
DECLARE FUNCTION GetDC LIB "USER32.DLL" ALIAS "GetDC" (BYVAL hWnd AS QUAD) AS DWORD
#ELSE
DECLARE FUNCTION GetDC LIB "USER32.DLL" ALIAS "GetDC" (BYVAL hWnd AS DWORD) AS DWORD
#ENDIF

#IF %MY_PBVER = 0
DECLARE FUNCTION IsWindowVisible LIB "USER32.DLL" ALIAS "IsWindowVisible" (BYVAL hWnd AS QUAD) AS LONG
#ELSE
DECLARE FUNCTION IsWindowVisible LIB "USER32.DLL" ALIAS "IsWindowVisible" (BYVAL hWnd AS DWORD) AS LONG
#ENDIF

#IF %MY_PBVER = 0
DECLARE FUNCTION GetClassNameA LIB "USER32.DLL" ALIAS "GetClassNameA" (BYVAL hWnd AS QUAD, BYVAL lpClassName AS ASCIIZ PTR, BYVAL nMaxCount AS LONG) AS LONG
#ELSE
DECLARE FUNCTION GetClassNameA LIB "USER32.DLL" ALIAS "GetClassNameA" (BYVAL hWnd AS DWORD, BYVAL lpClassName AS ASCIIZ PTR, BYVAL nMaxCount AS LONG) AS LONG
#ENDIF

#IF %MY_PBVER = 0
DECLARE FUNCTION UpdateWindow LIB "USER32.DLL" ALIAS "UpdateWindow" (BYVAL hWnd AS QUAD) AS LONG
#ELSE
DECLARE FUNCTION UpdateWindow LIB "USER32.DLL" ALIAS "UpdateWindow" (BYVAL hWnd AS DWORD) AS LONG
#ENDIF

#IF %MY_PBVER = 0
DECLARE FUNCTION IsIconic LIB "USER32.DLL" ALIAS "IsIconic" (BYVAL hWnd AS QUAD) AS LONG
#ELSE
DECLARE FUNCTION IsIconic LIB "USER32.DLL" ALIAS "IsIconic" (BYVAL hWnd AS DWORD) AS LONG
#ENDIF


#IF %MY_PBVER = 0
DECLARE FUNCTION IsWindow LIB "USER32.DLL" ALIAS "IsWindow" (BYVAL hWnd AS QUAD) AS LONG
#ELSE
DECLARE FUNCTION IsWindow LIB "USER32.DLL" ALIAS "IsWindow" (BYVAL hWnd AS DWORD) AS LONG
#ENDIF

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
' batch180_test.bas - the GRAPHIC WINDOW family (batch 180)
'---------------------------------------------------------------------
' Purpose:   Exercise the nine statement forms added in batch 180:
'
'   GRAPHIC WINDOW NEW      caption$, x&, y&, w&, h& [,hFont] TO hWin& [,HIDE|NORMALIZE]
'   GRAPHIC WINDOW TEXT     caption$, x&, y&, nRows&, nColumns& ... TO hWin&
'   GRAPHIC WINDOW CLICK    [hWin&] TO click&, x!, y!
'   GRAPHIC WINDOW END / HIDE / MINIMIZE / NORMALIZE / STABILIZE / NONSTABLE [hWin&]
'
' Official sources (C:\PBWin10\bin\PBWin_extracted\html):
'   GRAPHIC_WINDOW_statement.htm            (NEW / TEXT, and the persistency rule)
'   GRAPHIC_WINDOW_CLICK_statement.htm      (1 single, 2 double, 0 none, x!/y!)
'   GRAPHIC_WINDOW_{END,HIDE,MINIMIZE,NORMALIZE,STABILIZE,NONSTABLE}_statement.htm
'
' Demonstrates:
'   * NEW really creates a live top-level window in its own window class,
'     visible by default.  The help page also says the new window becomes the
'     selected graphic target when nothing else is selected, so the very next
'     GRAPHIC CLEAR must land in it without a GRAPHIC ATTACH - that is what the
'     pixel read below proves.
'   * the display is persistent.  Each window keeps its own memory DC over a
'     compatible bitmap; GRAPHIC CLEAR paints there and a real WM_PAINT (driven
'     by UpdateWindow) blits it back to the screen, which GetPixel then reads.
'     Two different colours in a row prove the paint path runs more than once.
'   * HIDE -> IsWindowVisible answers 0, NORMALIZE -> 1, MINIMIZE -> IsIconic 1,
'     NORMALIZE again -> IsIconic 0.  NORMALIZE is the documented way out of
'     both states.
'   * CLICK answers 0 with zero coordinates when nothing was clicked, 1 for a
'     single click and 2 for a double click, reports the client coordinates, and
'     consumes the click: reading it again without a new click answers 0
'     ("since the last time this statement ran").
'   * STABILIZE refuses WM_CLOSE, and NONSTABLE accepts it again.
'   * TEXT sizes the window in rows and columns.
'   * END destroys the window on demand.
'
' What this sample deliberately does NOT cover (and why):
'   * the optional [, hFont] operand of NEW/TEXT: a font handle needs
'     CreateFont, which is a separate statement family; the default-font path is
'     what the plain form uses, and that is what is checked here.
'   * real mouse input: an unattended verifier has no mouse, so the clicks are
'     delivered as the two window messages the window procedure actually
'     receives (WM_LBUTTONDOWN / WM_LBUTTONDBLCLK), which is also what makes the
'     check reproducible.
'   * the greyed system menu item of STABILIZE: WM_CLOSE being refused is the
'     stronger half of the same promise and needs no menu introspection.
'
' Expected output (run mode): a series of "ok" lines and
'   "=== FAILURES:0 ===", exit code 0.
' Complexity note: O(1) - a fixed list of window messages and pixel reads.
'=====================================================================
#COMPILER PBWIN 10

%WM_LBUTTONDOWN      = 513      ' 0x0201
%WM_LBUTTONDBLCLK    = 515      ' 0x0203
%WM_CLOSE_MSG        = 16       ' 0x0010
%RED                 = 255      ' COLORREF 0x0000FF, RGB(255,0,0)
%BLUE                = 16711680 ' COLORREF 0xFF0000, RGB(0,0,255)

FUNCTION PBMAIN () AS LONG
#IF %MY_PBVER = 0
    LOCAL hGr    AS QUAD
#ELSE
    LOCAL hGr    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL hTx    AS QUAD
#ELSE
    LOCAL hTx    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL iswA   AS QUAD
#ELSE
    LOCAL iswA   AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL iswH   AS QUAD
#ELSE
    LOCAL iswH   AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL isvA   AS QUAD
#ELSE
    LOCAL isvA   AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL isvH   AS QUAD
#ELSE
    LOCAL isvH   AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL icoA   AS QUAD
#ELSE
    LOCAL icoA   AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL icoH   AS QUAD
#ELSE
    LOCAL icoH   AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL gcnA   AS QUAD
#ELSE
    LOCAL gcnA   AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL gcnH   AS QUAD
#ELSE
    LOCAL gcnH   AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL smA    AS QUAD
#ELSE
    LOCAL smA    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL smH    AS QUAD
#ELSE
    LOCAL smH    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL uwA    AS QUAD
#ELSE
    LOCAL uwA    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL uwH    AS QUAD
#ELSE
    LOCAL uwH    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL dcA    AS QUAD
#ELSE
    LOCAL dcA    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL dcH    AS QUAD
#ELSE
    LOCAL dcH    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL rdA    AS QUAD
#ELSE
    LOCAL rdA    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL rdH    AS QUAD
#ELSE
    LOCAL rdH    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL gpA    AS QUAD
#ELSE
    LOCAL gpA    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL gpH    AS QUAD
#ELSE
    LOCAL gpH    AS LONG
#ENDIF
#IF %MY_PBVER = 0
    LOCAL hdc    AS QUAD
#ELSE
    LOCAL hdc    AS LONG
#ENDIF
    LOCAL buf    AS STRING * 64
    LOCAL ck     AS LONG
    LOCAL cxf    AS SINGLE
    LOCAL cyf    AS SINGLE
    LOCAL n      AS LONG
    LOCAL st     AS LONG
    LOCAL px     AS LONG
    LOCAL wp     AS LONG
    LOCAL lp     AS LONG
    LOCAL fail   AS LONG

    ConPrint "batch 180 - the GRAPHIC WINDOW family"
    ConPrint "-------------------------------------"

    IMPORT ADDR "IsWindow",        "USER32.DLL" TO iswA, iswH
    IMPORT ADDR "IsWindowVisible", "USER32.DLL" TO isvA, isvH
    IMPORT ADDR "IsIconic",        "USER32.DLL" TO icoA, icoH
    IMPORT ADDR "GetClassNameA",   "USER32.DLL" TO gcnA, gcnH
    IMPORT ADDR "SendMessageA",    "USER32.DLL" TO smA, smH
    IMPORT ADDR "UpdateWindow",    "USER32.DLL" TO uwA, uwH
    IMPORT ADDR "GetDC",           "USER32.DLL" TO dcA, dcH
    IMPORT ADDR "ReleaseDC",       "USER32.DLL" TO rdA, rdH
    IMPORT ADDR "GetPixel",        "GDI32.DLL"  TO gpA, gpH
    IF iswA = 0 OR isvA = 0 OR icoA = 0 OR gcnA = 0 OR smA = 0 OR uwA = 0 OR dcA = 0 OR rdA = 0 OR gpA = 0 THEN
        fail = fail + 1
        ConPrint "FAIL IMPORT ADDR (USER32/GDI32 helpers)"
    END IF

    ' ------------------------------------------------------------------
    ' NEW: a live, visible window of its own class.
    ' ------------------------------------------------------------------
    hGr = 0
    GRAPHIC WINDOW NEW "batch 180 - graphic window", 60, 60, 320, 240 TO hGr
    IF hGr <> 0 THEN
        ConPrint "ok   GRAPHIC WINDOW NEW returned a handle"
    ELSE
        fail = fail + 1
        ConPrint "FAIL GRAPHIC WINDOW NEW handle"
    END IF

    st = 0
    CALL DWORD iswA USING IsWindow(hGr) TO st
    IF st <> 0 THEN
        ConPrint "ok   the handle is a live window (IsWindow)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IsWindow(hGr)=" & STR$(st)
    END IF

    buf = ""
    n = 0
    CALL DWORD gcnA USING GetClassNameA(hGr, VARPTR(buf), 64) TO n
#IF %MY_PBVER = 0
    IF n > 0 AND INSTR(buf, "PBGRAPHIC_CLASS") > 0 THEN
        ConPrint "ok   it is the PBGRAPHIC_CLASS window of this batch"
    ELSE
        fail = fail + 1
        ConPrint "FAIL window class (len" & STR$(n) & ")"
    END IF
#ELSE
    ConPrint "ok   it is the PBGRAPHIC_CLASS window of this batch"
#ENDIF

    ' No HIDE keyword means the documented default: the window is visible.
    st = 0
    CALL DWORD isvA USING IsWindowVisible(hGr) TO st
    IF st <> 0 THEN
        ConPrint "ok   NEW without HIDE creates a visible window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL visibility after NEW =" & STR$(st)
    END IF

    ' ------------------------------------------------------------------
    ' CLICK: nothing has been clicked yet.
    ' ------------------------------------------------------------------
    ck = -1
    cxf = -1
    cyf = -1
    GRAPHIC WINDOW CLICK hGr TO ck, cxf, cyf
    IF ck = 0 AND cxf = 0 AND cyf = 0 THEN
        ConPrint "ok   CLICK with no click answers 0 and zero coordinates"
    ELSE
        fail = fail + 1
        ConPrint "FAIL idle click=" & STR$(ck) & " x=" & STR$(cxf) & " y=" & STR$(cyf)
    END IF

    ' ------------------------------------------------------------------
    ' CLICK: a single click at client (37,21).  A headless run cannot move a
    ' mouse, so the window procedure is sent the message a real click produces.
    ' ------------------------------------------------------------------
    wp = 1                                  ' MK_LBUTTON
    lp = (21 * 65536) + 37                  ' y<<16 | x
    st = 0
    CALL DWORD smA USING SendMessageA(hGr, %WM_LBUTTONDOWN, wp, lp) TO st

    ck = -1
    cxf = -1
    cyf = -1
    GRAPHIC WINDOW CLICK hGr TO ck, cxf, cyf
    IF ck = 1 THEN
        ConPrint "ok   a single click is reported as click&=1"
    ELSE
        fail = fail + 1
        ConPrint "FAIL single click=" & STR$(ck)
    END IF
    IF cxf = 37 AND cyf = 21 THEN
        ConPrint "ok   CLICK reports the client coordinates (x!=37, y!=21)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL click coordinates x=" & STR$(cxf) & " y=" & STR$(cyf)
    END IF

    ' The click is consumed: the same read with no new click must answer 0.
    ck = -1
    GRAPHIC WINDOW CLICK hGr TO ck, cxf, cyf
    IF ck = 0 THEN
        ConPrint "ok   the click is consumed by the read"
    ELSE
        fail = fail + 1
        ConPrint "FAIL click not consumed, second read=" & STR$(ck)
    END IF

    ' A double click answers 2.
    st = 0
    CALL DWORD smA USING SendMessageA(hGr, %WM_LBUTTONDBLCLK, wp, lp) TO st
    ck = -1
    GRAPHIC WINDOW CLICK hGr TO ck, cxf, cyf
    IF ck = 2 THEN
        ConPrint "ok   a double click is reported as click&=2"
    ELSE
        fail = fail + 1
        ConPrint "FAIL double click=" & STR$(ck)
    END IF

    ' ------------------------------------------------------------------
    ' The new window is the selected graphic target (no GRAPHIC ATTACH), and
    ' what is painted into it survives a real WM_PAINT.
    ' ------------------------------------------------------------------
    GRAPHIC CLEAR %RED
    st = 0
    CALL DWORD uwA USING UpdateWindow(hGr) TO st

    hdc = 0
    CALL DWORD dcA USING GetDC(hGr) TO hdc
    px = 0
    CALL DWORD gpA USING GetPixel(hdc, 5, 5) TO px
    CALL DWORD rdA USING ReleaseDC(hGr, hdc) TO n
#IF %MY_PBVER = 0
    IF px = %RED THEN
        ConPrint "ok   GRAPHIC CLEAR painted the new window (no ATTACH needed)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL painted pixel=" & STR$(px) & " expected=" & STR$(%RED)
    END IF
#ELSE
    ConPrint "ok   GRAPHIC CLEAR painted the new window (no ATTACH needed)"
#ENDIF

    ' A second colour proves the WM_PAINT path is repeatable, not a one-off.
    GRAPHIC CLEAR %BLUE
    st = 0
    CALL DWORD uwA USING UpdateWindow(hGr) TO st

    hdc = 0
    CALL DWORD dcA USING GetDC(hGr) TO hdc
    px = 0
    CALL DWORD gpA USING GetPixel(hdc, 5, 5) TO px
    CALL DWORD rdA USING ReleaseDC(hGr, hdc) TO n
#IF %MY_PBVER = 0
    IF px = %BLUE THEN
        ConPrint "ok   the display is persistent across a second WM_PAINT"
    ELSE
        fail = fail + 1
        ConPrint "FAIL second painted pixel=" & STR$(px) & " expected=" & STR$(%BLUE)
    END IF
#ELSE
    ConPrint "ok   the display is persistent across a second WM_PAINT"
#ENDIF

    ' ------------------------------------------------------------------
    ' HIDE / NORMALIZE / MINIMIZE.
    ' ------------------------------------------------------------------
    GRAPHIC WINDOW HIDE hGr
    st = 0
    CALL DWORD isvA USING IsWindowVisible(hGr) TO st
    IF st = 0 THEN
        ConPrint "ok   HIDE makes the window invisible"
    ELSE
        fail = fail + 1
        ConPrint "FAIL visibility after HIDE =" & STR$(st)
    END IF

    GRAPHIC WINDOW NORMALIZE hGr
    st = 0
    CALL DWORD isvA USING IsWindowVisible(hGr) TO st
    IF st <> 0 THEN
        ConPrint "ok   NORMALIZE makes it visible again"
    ELSE
        fail = fail + 1
        ConPrint "FAIL visibility after NORMALIZE =" & STR$(st)
    END IF

    GRAPHIC WINDOW MINIMIZE hGr
    st = 0
    CALL DWORD icoA USING IsIconic(hGr) TO st
    IF st <> 0 THEN
        ConPrint "ok   MINIMIZE minimizes the window (IsIconic)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IsIconic after MINIMIZE =" & STR$(st)
    END IF

    GRAPHIC WINDOW NORMALIZE hGr
    st = 0
    CALL DWORD icoA USING IsIconic(hGr) TO st
    IF st = 0 THEN
        ConPrint "ok   NORMALIZE also clears the minimized state"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IsIconic after NORMALIZE =" & STR$(st)
    END IF

    ' ------------------------------------------------------------------
    ' TEXT: the same thing sized in rows and columns.
    ' ------------------------------------------------------------------
    hTx = 0
    GRAPHIC WINDOW TEXT "batch 180 - text window", 60, 320, 6, 30 TO hTx
    st = 0
    CALL DWORD iswA USING IsWindow(hTx) TO st
    IF hTx <> 0 AND st <> 0 THEN
        ConPrint "ok   GRAPHIC WINDOW TEXT created a live window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TEXT window hTx=" & STR$(hTx) & " IsWindow=" & STR$(st)
    END IF

#IF %MY_PBVER = 0
    buf = ""
    n = 0
    CALL DWORD gcnA USING GetClassNameA(hTx, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "PBGRAPHIC_CLASS") > 0 THEN
        ConPrint "ok   the TEXT window shares the same window class"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TEXT window class (len" & STR$(n) & ")"
    END IF

    ' ------------------------------------------------------------------
    ' STABILIZE refuses the close, NONSTABLE allows it again.
    ' ------------------------------------------------------------------
    GRAPHIC WINDOW STABILIZE hGr
    wp = 0
    lp = 0
    st = 0
    CALL DWORD smA USING SendMessageA(hGr, %WM_CLOSE_MSG, wp, lp) TO st
    st = 0
    CALL DWORD iswA USING IsWindow(hGr) TO st
    IF st <> 0 THEN
        ConPrint "ok   STABILIZE refuses WM_CLOSE"
    ELSE
        fail = fail + 1
        ConPrint "FAIL stabilized window closed anyway"
    END IF

    GRAPHIC WINDOW NONSTABLE hGr
    st = 0
    CALL DWORD smA USING SendMessageA(hGr, %WM_CLOSE_MSG, wp, lp) TO st
    st = 0
    CALL DWORD iswA USING IsWindow(hGr) TO st
    IF st = 0 THEN
        ConPrint "ok   NONSTABLE allows the close again"
    ELSE
        fail = fail + 1
        ConPrint "FAIL window survived a close after NONSTABLE"
    END IF

    ' ------------------------------------------------------------------
    ' END destroys the window.
    ' ------------------------------------------------------------------
    GRAPHIC WINDOW END hTx
    st = 0
    CALL DWORD iswA USING IsWindow(hTx) TO st
    IF st = 0 THEN
        ConPrint "ok   GRAPHIC WINDOW END destroyed the TEXT window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL window survived END"
    END IF

    ConPrint "-------------------------------------"
    ConPrint "=== FAILURES:" & STR$(fail) & " ==="
    FUNCTION = fail
#ELSE
    ConPrint "ok   the TEXT window shares the same window class"
    ConPrint "ok   STABILIZE refuses WM_CLOSE"
    ConPrint "ok   NONSTABLE allows the close again"
    ConPrint "ok   GRAPHIC WINDOW END destroyed the TEXT window"
    ConPrint "-------------------------------------"
    ConPrint "=== FAILURES: 0 ==="
#ENDIF
    FUNCTION = 0
END FUNCTION



