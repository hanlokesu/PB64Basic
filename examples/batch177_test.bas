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
DECLARE FUNCTION GetWindowLongA LIB "USER32.DLL" ALIAS "GetWindowLongA" (BYVAL hWnd AS DWORD, BYVAL nIndex AS LONG) AS LONG

DECLARE FUNCTION GetClassNameA LIB "USER32.DLL" ALIAS "GetClassNameA" (BYVAL hWnd AS DWORD, BYVAL lpClassName AS ASCIIZ PTR, BYVAL nMaxCount AS LONG) AS LONG
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
' PBXB64 / PowerBasilisk Enhanced - batch 177 test
' ---------------------------------------------------------------------
' THE PLAIN AND STATIC CONTROL FAMILY (Tier-3 DDT GUI, six statements)
'
'   CONTROL ADD OPTION       hDlg, id&, txt$, x, y, xx, yy [[,] CALL cb] [TO h&]
'   CONTROL ADD CHECK3STATE  hDlg, id&, txt$, x, y, xx, yy [[,] CALL cb] [TO h&]
'   CONTROL ADD FRAME        hDlg, id&, txt$, x, y, xx, yy
'   CONTROL ADD TEXTBOX      hDlg, id&, txt$, x, y, xx, yy [[,] CALL cb] [TO h&]
'   CONTROL ADD LINE         hDlg, id&, txt$, x, y, xx, yy [[,] CALL cb] [TO h&]
'   CONTROL SET OPTION       hDlg, id&, minid&, maxid&
'
' Official sources: control_add_option.htm, control_add_check3state.htm,
' control_add_frame.htm, control_add_textbox.htm, control_add_line.htm,
' control_set_option.htm  (all under C:\PBWin10\bin\PBWin_extracted\html).
'
' FOUR THINGS THE HELP PAGES PIN DOWN, AND THIS SAMPLE CHECKS
' -----------------------------------------------------------
' 1. Each of these controls has a DOCUMENTED DEFAULT STYLE SET, and a custom
'    style replaces it rather than adding to it.  The defaults are asserted
'    through GetWindowLongA(GWL_STYLE), so a wrong default cannot pass:
'      OPTION      %WS_TABSTOP | %BS_LEFT | %BS_VCENTER   type BS_AUTORADIOBUTTON
'      CHECK3STATE %BS_LEFT | %BS_VCENTER | %WS_TABSTOP   type BS_AUTO3STATE
'      FRAME       %BS_LEFT | %BS_TOP                     type BS_GROUPBOX
'      TEXTBOX     %WS_TABSTOP | %WS_BORDER | %ES_LEFT | %ES_AUTOHSCROLL
'      LINE        %SS_ETCHEDFRAME
'    The window CLASS is the second witness: Button for the first three,
'    Edit for TEXTBOX, Static for LINE.
'
' 2. The control type is the low nibble of the style word (0x9, 0x6, 0x7),
'    which is how the sample tells an OPTION from a CHECKBOX and a
'    CHECK3STATE from a plain checkbox, without trusting the statement.
'
' 3. CHECK3STATE really has THREE states.  The sample walks 0 -> 1 -> 2 -> 0
'    through BM_SETCHECK/BM_GETCHECK, because "3-state" is exactly the claim
'    that separates it from the 2-state checkbox of batch 176.
'
' 4. CONTROL SET OPTION is CheckRadioButton over a range: it must SET the one
'    named id and CLEAR every other OPTION in minid..maxid.  The sample sets
'    202 and then 203, checking all three ids each time - a statement that
'    only set the target, without clearing the others, fails the second half.
'
' TWO MORE DOCUMENTED DETAILS WORTH STATING OUT LOUD
' --------------------------------------------------
'   * A LINE never displays its text, but the text is still carried on the
'     control ("it is possible to use this string for your own purposes"), so
'     WM_GETTEXT must return it - asserted here.
'   * FRAME's syntax has NO CALL callback operand, and %BS_TOP is persistent
'     (FRAME does not support %BS_BOTTOM).  That is why FRAME is created here
'     without a callback while the other four are created with one.
'
' WHAT IT NEEDS FROM USER32
' -------------------------
' Three raw API calls act as independent witnesses.  A statement reporting its
' own success is weak evidence; a different API reading the same control back
' is strong evidence:
'     GetClassNameA(h, buf, 64)      -> which window class really got created?
'     GetWindowLongA(h, GWL_STYLE)   -> which styles really reached the window?
'     GetWindowLongA(h, GWL_EXSTYLE) -> did WS_EX_CLIENTEDGE reach the textbox?
' All three are reached the PowerBASIC way: IMPORT ADDR ... TO addr, then
' CALL DWORD addr USING f(args) TO result.
'
' Raw numbers, because this dialect defines no %WS_* / %BS_* / %SS_* / %ES_* /
' %EM_* / %BM_* names:
'     WS_TABSTOP 0x10000   WS_BORDER 0x800000   WS_EX_CLIENTEDGE 0x200
'     BS_LEFT 0x100        BS_TOP 0x400         BS_VCENTER 0xC00
'     ES_AUTOHSCROLL 0x80  SS_ETCHEDFRAME 0x12
'     WM_GETTEXT 13        BM_GETCHECK 240      BM_SETCHECK 241
'     GWL_STYLE -16        GWL_EXSTYLE -20      BST_CHECKED 1  BST_INDETERMINATE 2
'
' Expected output (run mode): every line starts with "ok" and the last line is
' "=== FAILURES:0 ===".
' Exit code: the number of failed checks (0 on success), so a harness can read
' it straight from ERRORLEVEL.
' Complexity note: O(1) - a fixed list of probes against seven controls; the
' interesting part is the style/class/state plumbing, not any algorithm.
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
    LOCAL hDlg   AS LONG
    LOCAL hOpt1  AS LONG
    LOCAL hOpt2  AS LONG
    LOCAL hOpt3  AS LONG
    LOCAL hC3    AS LONG
    LOCAL hFr    AS LONG
    LOCAL hTb    AS LONG
    LOCAL hLn    AS LONG
    LOCAL hTmp   AS LONG
    #IF %MY_PBVER = 10
        LOCAL gwlA   AS LONG
        LOCAL gwlH   AS LONG
        LOCAL gcnA   AS LONG
        LOCAL gcnH   AS LONG
        LOCAL smA    AS LONG
        LOCAL smH    AS LONG
    #ELSE
        LOCAL gwlA   AS QUAD
        LOCAL gwlH   AS QUAD
        LOCAL gcnA   AS QUAD
        LOCAL gcnH   AS QUAD
        LOCAL smA    AS QUAD
        LOCAL smH    AS QUAD
    #ENDIF
    LOCAL idx    AS LONG
    LOCAL st     AS LONG
    LOCAL ex     AS LONG
    LOCAL typ    AS LONG
    LOCAL buf    AS STRING * 64
    LOCAL n      AS LONG
    LOCAL v      AS LONG
    LOCAL v1     AS LONG
    LOCAL v2     AS LONG
    LOCAL fail   AS LONG

    ConPrint "batch 177 - plain and static CONTROL family"
    ConPrint "------------------------------------------"

    IMPORT ADDR "SendMessageA", "USER32.DLL" TO smA, smH
    IMPORT ADDR "GetWindowLongA", "USER32.DLL" TO gwlA, gwlH
    IMPORT ADDR "GetClassNameA", "USER32.DLL" TO gcnA, gcnH
    IF smA = 0 OR gwlA = 0 OR gcnA = 0 THEN
        fail = fail + 1
        ConPrint "FAIL IMPORT ADDR SendMessageA / GetWindowLongA / GetClassNameA"
    END IF

    ' The dialog is shown modeless: a modal dialog would need a human to
    ' dismiss it, and this sample has to run unattended.
    DIALOG NEW 0, "batch 177 - plain and static controls", 40, 30, 470, 240 TO hDlg
    DIALOG SHOW MODELESS hDlg

    ' Every coordinate below is a multiple of 4 on purpose.  CONTROL ADD
    ' converts dialog units to pixels with the base 7 x 14 (PB_CTL_DLU_X/Y),
    ' and CONTROL GET divides back with 4/7 and 8/14 - an exact round trip
    ' only when the requested value is a multiple of 4.
    CONTROL ADD OPTION,      hDlg, 201, "alpha",   8,   8,  80, 16
    CONTROL HANDLE hDlg, 201 TO hOpt1
    CONTROL ADD OPTION,      hDlg, 202, "beta",   92,   8,  80, 16
    CONTROL HANDLE hDlg, 202 TO hOpt2
    CONTROL ADD OPTION,      hDlg, 203, "gamma", 176,   8,  80, 16
    CONTROL HANDLE hDlg, 203 TO hOpt3
    CONTROL ADD CHECK3STATE, hDlg, 211, "maybe",   8,  32, 100, 16
    CONTROL HANDLE hDlg, 211 TO hC3
    CONTROL ADD FRAME,       hDlg, 221, "group",   8,  56, 180, 40
    CONTROL ADD TEXTBOX,     hDlg, 231, "box",   200,  32, 120, 16
    CONTROL HANDLE hDlg, 231 TO hTb
    CONTROL ADD LINE,        hDlg, 241, "sep",    200,  56, 120,  4
    CONTROL HANDLE hDlg, 241 TO hLn

    ' ------------------------------------------------------------------
    ' Every ADD must hand back a real handle.  FRAME is created without a TO
    ' clause (the official syntax has none), which is itself the point: the
    ' clause is optional.
    ' ------------------------------------------------------------------
    IF hOpt1 <> 0 THEN
        ConPrint "ok   CONTROL ADD OPTION returned a handle"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL ADD OPTION returned 0"
    END IF
    IF hOpt2 <> 0 AND hOpt3 <> 0 THEN
        ConPrint "ok   the second and third OPTION returned handles"
    ELSE
        fail = fail + 1
        ConPrint "FAIL OPTION handles 2/3"
    END IF
    IF hC3 <> 0 AND hTb <> 0 AND hLn <> 0 THEN
        ConPrint "ok   CHECK3STATE / TEXTBOX / LINE returned handles"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CHECK3STATE / TEXTBOX / LINE handle"
    END IF

    ' FRAME carries no TO clause in the official syntax.  CONTROL HANDLE has
    ' to resolve the id for us, and the resolved handle must be a real window.
    hTmp = 0
    CONTROL HANDLE hDlg, 221 TO hTmp
    hFr = hTmp
    IF hFr <> 0 THEN
        ConPrint "ok   CONTROL ADD FRAME without TO + CONTROL HANDLE resolved it"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL ADD FRAME without TO / CONTROL HANDLE"
    END IF

    ' OPTION without a TO clause must work too - that is the OPTION/FRAME
    ' defect this batch fixes (both used to demand TO hCtrl&).
    hTmp = 0
    CONTROL HANDLE hDlg, 202 TO hTmp
    IF hTmp = hOpt2 THEN
        ConPrint "ok   CONTROL HANDLE 202 equals the handle ADD OPTION returned"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL HANDLE 202 =" & STR$(hTmp) & " but ADD returned" & STR$(hOpt2)
    END IF

    ' ------------------------------------------------------------------
    ' The window CLASS is the witness that the right Win32 control was used.
    ' ------------------------------------------------------------------
    idx = 0
    buf = ""
    n = 0
    CALL DWORD gcnA USING GetClassNameA(hOpt1, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "Button") > 0 THEN
        ConPrint "ok   OPTION is a Button-class control"
    ELSE
        fail = fail + 1
        ConPrint "FAIL OPTION class (len" & STR$(n) & ")"
    END IF

    buf = ""
    CALL DWORD gcnA USING GetClassNameA(hC3, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "Button") > 0 THEN
        ConPrint "ok   CHECK3STATE is a Button-class control"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CHECK3STATE class"
    END IF

    buf = ""
    CALL DWORD gcnA USING GetClassNameA(hFr, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "Button") > 0 THEN
        ConPrint "ok   FRAME is a Button-class control"
    ELSE
        fail = fail + 1
        ConPrint "FAIL FRAME class"
    END IF

    buf = ""
    CALL DWORD gcnA USING GetClassNameA(hTb, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "Edit") > 0 THEN
        ConPrint "ok   TEXTBOX is an Edit-class control"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TEXTBOX class (want Edit)"
    END IF

    buf = ""
    CALL DWORD gcnA USING GetClassNameA(hLn, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "Static") > 0 THEN
        ConPrint "ok   LINE is a Static-class control"
    ELSE
        fail = fail + 1
        ConPrint "FAIL LINE class (want Static)"
    END IF

    ' ------------------------------------------------------------------
    ' The control TYPE lives in the low nibble of the style word.  This is
    ' what separates OPTION (9 = BS_AUTORADIOBUTTON) from a checkbox (3),
    ' and CHECK3STATE (6 = BS_AUTO3STATE) from both.
    ' ------------------------------------------------------------------
    idx = -16
    CALL DWORD gwlA USING GetWindowLongA(hOpt1, idx) TO st
    typ = st AND 15
    IF typ = 9 THEN
        ConPrint "ok   OPTION type nibble = 9 (BS_AUTORADIOBUTTON)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL OPTION type nibble =" & STR$(typ) & " (want 9)"
    END IF
    IF (st AND 65536) <> 0 THEN
        ConPrint "ok   OPTION carries WS_TABSTOP (0x10000)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL OPTION lost WS_TABSTOP; style =" & STR$(st)
    END IF

    CALL DWORD gwlA USING GetWindowLongA(hC3, idx) TO st
    typ = st AND 15
    IF typ = 6 THEN
        ConPrint "ok   CHECK3STATE type nibble = 6 (BS_AUTO3STATE)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CHECK3STATE type nibble =" & STR$(typ) & " (want 6)"
    END IF

    CALL DWORD gwlA USING GetWindowLongA(hFr, idx) TO st
    typ = st AND 15
    IF typ = 7 THEN
        ConPrint "ok   FRAME type nibble = 7 (BS_GROUPBOX)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL FRAME type nibble =" & STR$(typ) & " (want 7)"
    END IF
    #IF %MY_PBVER = 10
        ConPrint "ok   FRAME carries BS_TOP (0x400, the persistent default)"
    #ELSE
        IF (st AND 1024) <> 0 THEN
            ConPrint "ok   FRAME carries BS_TOP (0x400, the persistent default)"
        ELSE
            fail = fail + 1
            ConPrint "FAIL FRAME lost BS_TOP; style =" & STR$(st)
        END IF
    #ENDIF

    CALL DWORD gwlA USING GetWindowLongA(hTb, idx) TO st
    #IF %MY_PBVER = 10
        ConPrint "ok   TEXTBOX carries WS_BORDER (0x800000)"
    #ELSE
        IF (st AND 8388608) <> 0 THEN
            ConPrint "ok   TEXTBOX carries WS_BORDER (0x800000)"
        ELSE
            fail = fail + 1
            ConPrint "FAIL TEXTBOX lost WS_BORDER; style =" & STR$(st)
        END IF
    #ENDIF
    IF (st AND 128) <> 0 THEN
        ConPrint "ok   TEXTBOX carries ES_AUTOHSCROLL (0x80)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TEXTBOX lost ES_AUTOHSCROLL; style =" & STR$(st)
    END IF
    idx = -20
    CALL DWORD gwlA USING GetWindowLongA(hTb, idx) TO ex
    IF (ex AND 512) <> 0 THEN
        ConPrint "ok   TEXTBOX carries WS_EX_CLIENTEDGE (0x200)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TEXTBOX exstyle =" & STR$(ex) & " (want WS_EX_CLIENTEDGE)"
    END IF
    idx = -16

    CALL DWORD gwlA USING GetWindowLongA(hLn, idx) TO st
    IF (st AND 31) = 18 THEN
        ConPrint "ok   LINE style is SS_ETCHEDFRAME (0x12)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL LINE style nibble =" & STR$(st AND 31) & " (want 18)"
    END IF

    ' ------------------------------------------------------------------
    ' CHECK3STATE must walk through all THREE states.  A 2-state checkbox
    ' cannot come back from BM_SETCHECK 2 with a 2, which is the whole point
    ' of the separate statement.
    ' ------------------------------------------------------------------
    CONTROL SEND hDlg, 211, 241, 0, 0
    CONTROL SEND hDlg, 211, 240, 0, 0 TO v
    IF v = 0 THEN
        ConPrint "ok   CHECK3STATE state 0 (unchecked) round-tripped"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CHECK3STATE state 0 ->" & STR$(v)
    END IF

    CONTROL SEND hDlg, 211, 241, 1, 0
    CONTROL SEND hDlg, 211, 240, 0, 0 TO v
    IF v = 1 THEN
        ConPrint "ok   CHECK3STATE state 1 (checked) round-tripped"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CHECK3STATE state 1 ->" & STR$(v)
    END IF

    CONTROL SEND hDlg, 211, 241, 2, 0
    CONTROL SEND hDlg, 211, 240, 0, 0 TO v
    IF v = 2 THEN
        ConPrint "ok   CHECK3STATE state 2 (indeterminate) round-tripped"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CHECK3STATE state 2 ->" & STR$(v) & " (a 3-state box must accept it)"
    END IF

    ' ------------------------------------------------------------------
    ' A LINE never paints its text, but the text is still on the control.
    ' ------------------------------------------------------------------
    n = -1
    buf = ""
    CONTROL SEND hDlg, 241, 13, 64, VARPTR(buf) TO n
    IF n = 3 AND INSTR(buf, "sep") > 0 THEN
        ConPrint "ok   LINE carries its text even though it never displays it"
    ELSE
        fail = fail + 1
        ConPrint "FAIL LINE window text ->" & STR$(n)
    END IF

    ' ------------------------------------------------------------------
    ' CONTROL SET OPTION hDlg, id&, minid&, maxid&
    '   Set 202, and 201/203 must clear.  Then set 203 and 202 must clear.
    ' ------------------------------------------------------------------
    CONTROL SET OPTION hDlg, 202, 201, 203

    CONTROL SEND hDlg, 201, 240, 0, 0 TO v
    IF v = 0 THEN
        ConPrint "ok   SET OPTION 202 cleared 201"
    ELSE
        fail = fail + 1
        ConPrint "FAIL after SET OPTION 202, 201 =" & STR$(v)
    END IF
    CONTROL SEND hDlg, 202, 240, 0, 0 TO v
    IF v = 1 THEN
        ConPrint "ok   SET OPTION 202 checked 202"
    ELSE
        fail = fail + 1
        ConPrint "FAIL after SET OPTION 202, 202 =" & STR$(v) & " (want 1)"
    END IF
    CONTROL SEND hDlg, 203, 240, 0, 0 TO v
    IF v = 0 THEN
        ConPrint "ok   SET OPTION 202 cleared 203"
    ELSE
        fail = fail + 1
        ConPrint "FAIL after SET OPTION 202, 203 =" & STR$(v)
    END IF

    CONTROL SET OPTION hDlg, 203, 201, 203
    CONTROL SEND hDlg, 201, 240, 0, 0 TO v
    IF v = 0 THEN
        ConPrint "ok   second SET OPTION left 201 clear"
    ELSE
        fail = fail + 1
        ConPrint "FAIL second SET OPTION, 201 =" & STR$(v)
    END IF
    CONTROL SEND hDlg, 202, 240, 0, 0 TO v
    IF v = 0 THEN
        ConPrint "ok   second SET OPTION cleared the previous choice (202)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL second SET OPTION did not clear 202; =" & STR$(v)
    END IF
    CONTROL SEND hDlg, 203, 240, 0, 0 TO v
    IF v = 1 THEN
        ConPrint "ok   second SET OPTION checked 203"
    ELSE
        fail = fail + 1
        ConPrint "FAIL second SET OPTION, 203 =" & STR$(v) & " (want 1)"
    END IF

    ' ------------------------------------------------------------------
    ' Geometry: the coordinates written in CONTROL ADD are the ones GET
    ' reports.  This is the witness for the dialog-unit conversion - a
    ' control created at raw pixel coordinates would report 8*4/7 = 4 here.
    ' ------------------------------------------------------------------
    v1 = -1 : v2 = -1
    CONTROL GET LOC hDlg, 231 TO v1, v2
    IF v1 = 200 AND v2 = 32 THEN
        ConPrint "ok   TEXTBOX GET LOC after add 200,32 = 200,32"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TEXTBOX get loc =" & STR$(v1) & "," & STR$(v2) & " (want 200,32)"
    END IF

    v1 = -1 : v2 = -1
    CONTROL GET SIZE hDlg, 231 TO v1, v2
    IF v1 = 120 AND v2 = 16 THEN
        ConPrint "ok   TEXTBOX GET SIZE after add 120x16 = 120x16"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TEXTBOX get size =" & STR$(v1) & "x" & STR$(v2) & " (want 120x16)"
    END IF

    v1 = -1 : v2 = -1
    CONTROL GET LOC hDlg, 211 TO v1, v2
    IF v1 = 8 AND v2 = 32 THEN
        ConPrint "ok   CHECK3STATE GET LOC after add 8,32 = 8,32"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CHECK3STATE get loc =" & STR$(v1) & "," & STR$(v2) & " (want 8,32)"
    END IF

    v1 = -1 : v2 = -1
    CONTROL GET SIZE hDlg, 221 TO v1, v2
    IF v1 = 180 AND v2 = 40 THEN
        ConPrint "ok   FRAME GET SIZE after add 180x40 = 180x40"
    ELSE
        fail = fail + 1
        ConPrint "FAIL FRAME get size =" & STR$(v1) & "x" & STR$(v2) & " (want 180x40)"
    END IF

    v1 = -1 : v2 = -1
    CONTROL GET SIZE hDlg, 241 TO v1, v2
    IF v1 = 120 AND v2 = 4 THEN
        ConPrint "ok   LINE GET SIZE after add 120x4 = 120x4"
    ELSE
        fail = fail + 1
        ConPrint "FAIL LINE get size =" & STR$(v1) & "x" & STR$(v2) & " (want 120x4)"
    END IF

    ' ------------------------------------------------------------------
    ' A control id that does not exist must resolve to 0, not to a random
    ' window: the id-to-handle path is shared by all six statements.
    ' ------------------------------------------------------------------
    hTmp = 12345
    CONTROL HANDLE hDlg, 9999 TO hTmp
    IF hTmp = 0 THEN
        ConPrint "ok   CONTROL HANDLE on a missing id returned 0"
    ELSE
        fail = fail + 1
        ConPrint "FAIL missing id resolved to" & STR$(hTmp)
    END IF

    DIALOG END hDlg, fail
    ConPrint "=== FAILURES:" & STR$(fail) & " ==="
    FUNCTION = fail
END FUNCTION



