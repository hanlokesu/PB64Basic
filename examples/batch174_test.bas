#COMPILE EXE

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
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

DECLARE FUNCTION GetMenu LIB "USER32.DLL" ALIAS "GetMenu" (BYVAL hWnd AS DWORD) AS DWORD
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
' batch174_test.bas - MENU ATTACH / MENU CONTEXT / MENU DRAW BAR
'---------------------------------------------------------------------
' Purpose
'   Exercise the three MENU statements added in batch 174 and, where the
'   official page makes a promise that a headless program can observe,
'   pin that promise down with an exact comparison:
'
'       MENU ATTACH   hMenu, hDlg  - attach a menu to a dialog, REPLACING
'                                    any existing menu, and redraw the
'                                    dialog to make room for it
'       MENU CONTEXT  hMenu, x&, y&, flags& TO CmdVar&
'                                  - floating popup menu; the id the user
'                                    picked comes back in CmdVar&
'       MENU DRAW BAR hDlg         - redraw a dialog's menu bar after the
'                                    menu was altered dynamically
'
'   How the promises are checked:
'
'     * "replacing any existing menu" - GetMenu(hDlg) is read back through
'       IMPORT ADDR / CALL DWORD and must equal the bar that was attached
'       last, exactly.  (This is the same mechanism batch022_test.bas uses
'       for GetTickCount; the batch 174 release also fixed that path, which
'       used to drop the callee's arguments.)
'     * "the dialog is redrawn to accommodate the new menu" - the dialog is
'       shown modeless, and DIALOG GET CLIENT must report a SMALLER client
'       height while a menu bar is attached, and the original height again
'       once the menu is removed.  The comparison is relative on purpose:
'       the menu bar's pixel height depends on the DPI and the theme.
'     * MENU CONTEXT: a real popup menu waits for the user, so it cannot be
'       part of an unattended run.  The null-handle case is the part that can
'       be: it must return at once and report 0 ("no selection").
'
' Demonstrates
'   * MENU NEW BAR / MENU NEW POPUP / MENU ADD STRING / MENU ADD POPUP
'   * MENU ATTACH hMenu, hDlg          (attach, then read the handle back)
'   * MENU ATTACH hMenu2, hDlg         (replacement of the existing menu)
'   * MENU ATTACH 0, hDlg              (removal: SetMenu(hDlg, NULL))
'   * MENU DRAW BAR hDlg               (before and after a dynamic change)
'   * MENU CONTEXT ... TO CmdVar&      (null handle -> 0, returns at once)
'
' Expected output (run mode)
'   one line per assertion, then
'   === FAILURES: 0 ===
'   (exit code 0)
'
' Notes
'   * This sample opens a MODELESS dialog briefly.  It runs no message loop
'     and waits for no key, so the verification harness can run it
'     unattended; DIALOG SHOW MODELESS is what makes the client-area
'     measurement meaningful (a hidden window is not laid out yet).
'   * Sample files are text; keep this one headless.
' Complexity note: O(1) - a fixed sequence of menu and dialog calls.
'=====================================================================
#COMPILER PBWIN 10
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF


FUNCTION PBMAIN () AS LONG
    LOCAL hDlg  AS LONG
    LOCAL hBar1 AS LONG
    LOCAL hBar2 AS LONG
    LOCAL hPop  AS LONG
    LOCAL hGot  AS LONG
    #IF %MY_PBVER = 10
        LOCAL gaddr AS LONG
        LOCAL ghndl AS LONG
    #ELSE
        LOCAL gaddr AS QUAD
        LOCAL ghndl AS QUAD
    #ENDIF
    LOCAL cw    AS LONG
    LOCAL ch0   AS LONG         ' client height with no menu
    LOCAL ch1   AS LONG         ' client height with bar1
    LOCAL ch2   AS LONG         ' client height with bar2
    LOCAL ch3   AS LONG         ' client height after removing the menu
    LOCAL cmd   AS LONG
    LOCAL fail  AS LONG

    DIALOG NEW 0, "batch174 MENU ATTACH probe", 0, 0, 400, 200 TO hDlg
    DIALOG SHOW MODELESS hDlg
    DIALOG GET CLIENT hDlg TO cw, ch0

    ' GetMenu from USER32 - the only way a PB program can read back which menu a
    ' dialog carries.
    #IF %MY_PBVER = 10
        ' PBWin: CALL DWORD cannot read GetMenu (returns 0 and breaks the
        ' dialog's client queries - probe174pb); use the direct DECLARE call.
        gaddr = 1
    #ELSE
        IMPORT ADDR "GetMenu", "USER32.DLL" TO gaddr, ghndl
    #ENDIF
    IF gaddr = 0 THEN
        fail = fail + 1
        ConPrint "FAIL IMPORT ADDR GetMenu failed - cannot verify attachment"
    END IF

    ' ---------------------------------------------------------------
    ' A fresh dialog carries no menu.
    ' ---------------------------------------------------------------
    hGot = 12345
    #IF %MY_PBVER = 10
        hGot = GetMenu(hDlg)
    #ELSE
        CALL DWORD gaddr USING GetMenu(hDlg) TO hGot
    #ENDIF
    IF hGot <> 0 THEN
        fail = fail + 1
        ConPrint "FAIL GetMenu before attach ->" & STR$(hGot) & " (expected 0, a bare dialog has no menu)"
    ELSE
        ConPrint "ok   GetMenu before attach  -> 0"
    END IF

    ' ---------------------------------------------------------------
    ' Build two distinct bars.  Each is one item so the bars are valid
    ' menus; bar1 also gets a popup subtree.
    ' ---------------------------------------------------------------
    hBar1 = 0
    MENU NEW BAR TO hBar1
    hBar2 = 0
    MENU NEW BAR TO hBar2
    hPop = 0
    MENU NEW POPUP TO hPop
    IF hBar1 = 0 OR hBar2 = 0 OR hPop = 0 THEN
        fail = fail + 1
        ConPrint "FAIL MENU NEW BAR/POPUP returned 0 (" & STR$(hBar1) & STR$(hBar2) & STR$(hPop) & ")"
    ELSE
        ConPrint "ok   MENU NEW BAR x2, NEW POPUP -> handles"
    END IF
    IF hBar1 = hBar2 THEN
        fail = fail + 1
        ConPrint "FAIL the two bars are the same handle"
    END IF

    MENU ADD STRING, hBar1, "File", 100, 0
    MENU ADD POPUP, hBar1, "View", hPop, 0
    MENU ADD STRING, hBar2, "Edit", 200, 0
    ConPrint "ok   MENU ADD STRING / ADD POPUP -> items added"

    ' ---------------------------------------------------------------
    ' ATTACH: GetMenu must report exactly the handle we attached, and the
    ' client area must shrink by the menu bar's height.
    ' ---------------------------------------------------------------
    MENU ATTACH hBar1, hDlg
    hGot = 0
    #IF %MY_PBVER = 10
        hGot = GetMenu(hDlg)
    #ELSE
        CALL DWORD gaddr USING GetMenu(hDlg) TO hGot
    #ENDIF
    IF hGot <> hBar1 THEN
        fail = fail + 1
        ConPrint "FAIL GetMenu after attach ->" & STR$(hGot) & " (expected the bar" & STR$(hBar1) & ")"
    ELSE
        ConPrint "ok   GetMenu after attach   -> the attached bar"
    END IF
    DIALOG GET CLIENT hDlg TO cw, ch1
    IF ch1 >= ch0 THEN
        fail = fail + 1
        ConPrint "FAIL client height with menu ->" & STR$(ch1) & " (expected < " & STR$(ch0) & ")"
    ELSE
        ConPrint "ok   client height shrinks by the menu bar (relative)"
    END IF

    ' ---------------------------------------------------------------
    ' REPLACEMENT: attaching the second bar must replace the first.
    ' ---------------------------------------------------------------
    MENU ATTACH hBar2, hDlg
    hGot = 0
    #IF %MY_PBVER = 10
        hGot = GetMenu(hDlg)
    #ELSE
        CALL DWORD gaddr USING GetMenu(hDlg) TO hGot
    #ENDIF
    IF hGot <> hBar2 THEN
        fail = fail + 1
        ConPrint "FAIL GetMenu after replace ->" & STR$(hGot) & " (expected the second bar" & STR$(hBar2) & ")"
    ELSE
        ConPrint "ok   MENU ATTACH replaced the first bar"
    END IF
    DIALOG GET CLIENT hDlg TO cw, ch2
    IF ch2 <> ch1 THEN
        fail = fail + 1
        ConPrint "FAIL client height changed on replacement ->" & STR$(ch2) & " (expected " & STR$(ch1) & ")"
    ELSE
        ConPrint "ok   same menu-bar height after replacement"
    END IF

    ' ---------------------------------------------------------------
    ' DRAW BAR: must repaint after a dynamic menu change, and must not
    ' disturb which menu is attached.
    ' ---------------------------------------------------------------
    MENU DRAW BAR hDlg
    MENU ADD STRING, hBar2, "More", 300, 0
    MENU DRAW BAR hDlg
    hGot = 0
    #IF %MY_PBVER = 10
        hGot = GetMenu(hDlg)
    #ELSE
        CALL DWORD gaddr USING GetMenu(hDlg) TO hGot
    #ENDIF
    IF hGot <> hBar2 THEN
        fail = fail + 1
        ConPrint "FAIL GetMenu after DRAW BAR ->" & STR$(hGot) & " (expected " & STR$(hBar2) & ")"
    ELSE
        ConPrint "ok   MENU DRAW BAR x2 kept the bar attached"
    END IF

    ' ---------------------------------------------------------------
    ' CONTEXT: a null handle must return at once and report 0.  A popup on
    ' a real menu handle is interactive by design and is NOT run here.
    ' ---------------------------------------------------------------
    cmd = 777
    MENU CONTEXT 0, 100, 100, 0 TO cmd
    IF cmd <> 0 THEN
        fail = fail + 1
        ConPrint "FAIL MENU CONTEXT null ->" & STR$(cmd) & " (expected 0, no selection)"
    ELSE
        ConPrint "ok   MENU CONTEXT null       -> 0, returned immediately"
    END IF

    ' ---------------------------------------------------------------
    ' REMOVAL: attaching 0 removes the menu; the client area must grow
    ' back to its original height.  This is the control that proves the
    ' height difference above really was the menu bar.
    ' ---------------------------------------------------------------
    MENU ATTACH 0, hDlg
    hGot = 0
    #IF %MY_PBVER = 10
        hGot = GetMenu(hDlg)
    #ELSE
        CALL DWORD gaddr USING GetMenu(hDlg) TO hGot
    #ENDIF
    IF hGot <> 0 THEN
        fail = fail + 1
        ConPrint "FAIL GetMenu after removal ->" & STR$(hGot) & " (expected 0)"
    ELSE
        ConPrint "ok   MENU ATTACH 0 removed the menu"
    END IF
    DIALOG GET CLIENT hDlg TO cw, ch3
    IF ch3 <> ch0 THEN
        fail = fail + 1
        ConPrint "FAIL client height after removal ->" & STR$(ch3) & " (expected " & STR$(ch0) & ")"
    ELSE
        ConPrint "ok   client height restored (relative)"
    END IF
    MENU DRAW BAR hDlg
    ConPrint "ok   MENU DRAW BAR on a menu-less dialog is harmless"

    IF gaddr <> 0 THEN
        #IF %MY_PBVER = 10
            REM (PBWin branch: nothing was imported)
        #ELSE
            IMPORT CLOSE ghndl
        #ENDIF
    END IF
    ConPrint "=== FAILURES:" & STR$(fail) & " ==="
    DIALOG END hDlg, fail
    FUNCTION = fail
END FUNCTION



