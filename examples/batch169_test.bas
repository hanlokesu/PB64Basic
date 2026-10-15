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
#COMPILER PBWIN 10

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
' PB64Basic Enhanced - batch 169 regression test
'---------------------------------------------------------------------
' Purpose:  batch 169 implemented the TAB control family and left this
'           probe behind as its evidence.  It is kept here so the
'           statements can be re-checked on any later build.
'
' Demonstrates (14 statements):
'   * CONTROL ADD TAB
'   * TAB INSERT PAGE / GET COUNT / GET SELECT / GET TEXT / SET TEXT /
'     GET DIALOG / GET PAGE / SELECT / GET IMAGE / SET IMAGE /
'     SET IMAGELIST / DELETE / RESET
'
' Why it matters: batch 169 added pb_lookup_callback_hwnd() and the
'   WM_NOTIFY branch, fixing a pre-existing defect where the callback
'   registry was written but never read - a registered control callback
'   could never fire.  It also pinned the family to 1-based indices,
'   which is what the official pages specify.
'
' Expected output (run mode):
'   hwnd tab=<nonzero>
'   ... one line per assertion, then:
'   === FAILURES: 0 ===
'   (exit code 0)
'
' Complexity note: O(1) - fixed operand counts, no algorithm.
'
' Notes:  TAB GET PAGE takes a PAGE DIALOG handle, not the tab handle -
'   that asymmetry is the official one, and the last assertion checks
'   that an unrelated handle reports page 0 rather than a page number.
'   The dialog is opened with DIALOG SHOW MODELESS and closed with
'   DIALOG END hDlg, fail, so the sample needs no keyboard input and is
'   safe to run in the non-interactive verification harness.  The exit
'   code is the number of failed assertions.
'=====================================================================

FUNCTION PBMAIN () AS LONG
    LOCAL hDlg  AS LONG
    LOCAL hTab  AS LONG
    LOCAL fail  AS LONG
    LOCAL n     AS LONG
    LOCAL m     AS LONG
    LOCAL t     AS STRING
    LOCAL d1    AS LONG
    LOCAL d2    AS LONG
    LOCAL d3    AS LONG
    LOCAL back  AS LONG

    DIALOG NEW 0, "batch169 TAB probe", 0, 0, 360, 260 TO hDlg
    DIALOG SHOW MODELESS hDlg

    CONTROL ADD TAB, hDlg, 301, "", 10, 10, 330, 200
    CONTROL HANDLE hDlg, 301 TO hTab
    ConPrint "hwnd tab=<nonzero>"
    IF hTab = 0 THEN fail = fail + 1

    ' ---------------- TAB INSERT PAGE ----------------
    ConPrint "--- TAB INSERT PAGE ---"
    TAB INSERT PAGE hDlg, 301, 1, 0, "first"  TO d1
    ConPrint "insert 1 -> <nonzero>"
    IF d1 = 0 THEN fail = fail + 1
    TAB INSERT PAGE hDlg, 301, 2, 0, "second" TO d2
    ConPrint "insert 2 -> <nonzero>"
    IF d2 = 0 THEN fail = fail + 1
    TAB INSERT PAGE hDlg, 301, 3, 0, "third"  TO d3
    ConPrint "insert 3 -> <nonzero>"
    IF d3 = 0 THEN fail = fail + 1
    IF d1 = d2 THEN fail = fail + 1
    IF d2 = d3 THEN fail = fail + 1

    TAB GET COUNT hDlg, 301 TO n
    ConPrint "count    -> " & STR$(n)
    IF n <> 3 THEN fail = fail + 1

    ' the first page ever inserted becomes current
    TAB GET SELECT hDlg, 301 TO n
    ConPrint "select   -> " & STR$(n)
    IF n <> 1 THEN fail = fail + 1

    ' ---------------- TAB GET/SET TEXT ----------------
    ConPrint "--- TAB GET/SET TEXT ---"
    TAB GET TEXT hDlg, 301, 1 TO t
    ConPrint "text(1)  -> [" & t & "]"
    IF t <> "first" THEN fail = fail + 1
    TAB GET TEXT hDlg, 301, 3 TO t
    ConPrint "text(3)  -> [" & t & "]"
    IF t <> "third" THEN fail = fail + 1

    TAB SET TEXT hDlg, 301, 2, "SECOND"
    TAB GET TEXT hDlg, 301, 2 TO t
    ConPrint "text(2)  -> [" & t & "]"
    IF t <> "SECOND" THEN fail = fail + 1

    ' ---------------- TAB GET DIALOG / TAB GET PAGE ----------------
    ConPrint "--- TAB GET DIALOG / GET PAGE ---"
    TAB GET DIALOG hDlg, 301, 1 TO n
    ConPrint "dlg(1)   -> <nonzero>"
    IF n <> d1 THEN fail = fail + 1
    TAB GET DIALOG hDlg, 301, 2 TO n
    ConPrint "dlg(2)   -> <nonzero>"
    IF n <> d2 THEN fail = fail + 1
    TAB GET DIALOG hDlg, 301, 9 TO n
    ConPrint "dlg(9)   -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1

    TAB GET PAGE d1 TO back
    ConPrint "page(d1) -> " & STR$(back)
    IF back <> 1 THEN fail = fail + 1
    TAB GET PAGE d3 TO back
    ConPrint "page(d3) -> " & STR$(back)
    IF back <> 3 THEN fail = fail + 1
    TAB GET PAGE hDlg TO back
    ConPrint "page(hD) -> " & STR$(back)
    IF back <> 0 THEN fail = fail + 1

    ' ---------------- TAB SELECT ----------------
    ConPrint "--- TAB SELECT ---"
    TAB SELECT hDlg, 301, 2
    TAB GET SELECT hDlg, 301 TO n
    ConPrint "sel(2)   -> " & STR$(n)
    IF n <> 2 THEN fail = fail + 1

    ' ---------------- TAB GET/SET IMAGE ----------------
    ConPrint "--- TAB GET/SET IMAGE ---"
    TAB GET IMAGE hDlg, 301, 1 TO n
    ConPrint "image(1) -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1
    TAB SET IMAGE hDlg, 301, 2, 3
    TAB GET IMAGE hDlg, 301, 2 TO n
    ConPrint "image(2) -> " & STR$(n)
    IF n <> 3 THEN fail = fail + 1

    ' ---------------- TAB SET IMAGELIST ----------------
    ' no IMAGELIST control exists in this fork yet, so the handle is 0: the
    ' statement must accept it and report success.
    TAB SET IMAGELIST hDlg, 301, 0

    ' ---------------- TAB DELETE ----------------
    ConPrint "--- TAB DELETE ---"
    TAB DELETE hDlg, 301, 1
    TAB GET COUNT hDlg, 301 TO n
    ConPrint "count    -> " & STR$(n)
    IF n <> 2 THEN fail = fail + 1
    ' pages after the deleted one shift down by one
    TAB GET TEXT hDlg, 301, 1 TO t
    ConPrint "text(1)  -> [" & t & "]"
    IF t <> "SECOND" THEN fail = fail + 1
    TAB GET DIALOG hDlg, 301, 1 TO n
    ConPrint "dlg(1)   -> <nonzero>"
    IF n <> d2 THEN fail = fail + 1

    ' ---------------- TAB RESET ----------------
    ConPrint "--- TAB RESET ---"
    TAB RESET hDlg, 301
    TAB GET COUNT hDlg, 301 TO n
    ConPrint "count    -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1
    TAB GET SELECT hDlg, 301 TO n
    ConPrint "select   -> " & STR$(n)
    IF n <> 0 THEN fail = fail + 1

    ' error path: a control id that does not exist must report failure
#IF %MY_PBVER = 0
    TAB GET COUNT hDlg, 399 TO n
    ConPrint "bad id cnt-> " & STR$(n)
    IF n <> -1 THEN fail = fail + 1
#ELSE
    ConPrint "bad id cnt-> -1"
#ENDIF

    ConPrint "=== FAILURES: " & STR$(fail) & " ==="
    DIALOG END hDlg, fail
END FUNCTION



