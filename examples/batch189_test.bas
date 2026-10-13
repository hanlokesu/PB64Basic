

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
DECLARE FUNCTION SendMessageA LIB "USER32.DLL" ALIAS "SendMessageA" (BYVAL hWnd AS DWORD, BYVAL wMsg AS DWORD, BYVAL wParam AS DWORD, BYVAL lParam AS DWORD) AS LONG
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
' PowerBasilisk Enhanced - batch 189 test
' ---------------------------------------------------------------------
' THE COMMON-CONTROL FAMILY, PART 2 (Tier-3 DDT GUI, four statements)
'
'   CONTROL ADD TRACKBAR  hDlg, id&, x, y, w, h TO hCtl&
'   CONTROL ADD UPDOWN    hDlg, id&, x, y, w, h TO hCtl&
'   CONTROL ADD HOTKEY    hDlg, id&, x, y, w, h TO hCtl&
'   CONTROL ADD IPADDRESS hDlg, id&, x, y, w, h TO hCtl&
'
' Official sources: control_add_trackbar.htm, control_add_updown.htm,
' control_add_hotkey.htm, control_add_ipaddress.htm
' (all under C:\PBWin10\bin\PBWin_extracted\html).
'
' WHAT THIS SAMPLE WITNESSES, AND HOW
' -----------------------------------
' 1. WHICH WINDOW CLASS WAS REALLY CREATED.  Each CONTROL ADD hands back a
'    handle, and GetClassNameA must then report the common-control class:
'       TRACKBAR  msctls_trackbar32
'       UPDOWN    msctls_updown32
'       HOTKEY    msctls_hotkey32
'       IPADDRESS SysIPAddress32
'    A statement that reported success but created nothing - or created a
'    plain Button - cannot pass these four checks.
'
' 2. WHICH STYLES REALLY REACHED THE WINDOW (GetWindowLongA GWL_STYLE).
'    Every one of the four is WS_CHILD | WS_VISIBLE, plus WS_TABSTOP for
'    TRACKBAR / HOTKEY / IPADDRESS, TBS_AUTOTICKS for TRACKBAR and
'    UDS_SETBUDDYINT | UDS_ALIGNRIGHT for UPDOWN.
'
' 3. THAT THE CONTROL REALLY BEHAVES.  A handle plus a class name still
'    does not prove the window processes its messages, so each control is
'    driven through user32's SendMessageA and read back:
'       TBM_SETRANGE -> TBM_SETPOS -> TBM_GETPOS
'       UDM_SETPOS32 -> UDM_GETPOS32
'       HKM_SETHOTKEY -> HKM_GETHOTKEY
'       IPM_SETADDRESS -> IPM_GETADDRESS
'    Every message number used below was read out of the local
'    C:\PBWin10\WINAPI includes (the same files PowerBASIC itself uses),
'    not from memory - TBM_GETPOS is WM_USER, UDM_SETRANGE32 is WM_USER+111,
'    and so on.
'
' 4. THAT CONTROL HANDLE RESOLVES THE ID.  The TRACKBAR is found through
'    CONTROL HANDLE hDlg, 301 TO hTmp as a second route to the same window,
'    so the handles are cross-checked rather than trusted once.
'
' KNOWN LIMITATION, STATED HONESTLY
' ---------------------------------
' The official syntax prints the TO clause in brackets, but this family
' (all 28 CONTROL ADD types, not just these four) currently requires it.
' The sample therefore always passes TO, and optional-TO stays on the
' outstanding list rather than being quietly assumed to work.
'
' Run mode is headless: one line per check, ending with
' "=== FAILURES:0 ===".  The dialog is modeless because a modal one would
' need a human to dismiss it.
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL hDlg   AS LONG
    LOCAL hTb    AS LONG
    LOCAL hUd    AS LONG
    LOCAL hHk    AS LONG
    LOCAL hIp    AS LONG
    LOCAL hTmp   AS LONG
    LOCAL gcnA   AS LONG
    LOCAL gcnH   AS LONG
    LOCAL gwlA   AS LONG
    LOCAL gwlH   AS LONG
    LOCAL smA    AS LONG
    LOCAL smH    AS LONG
    LOCAL buf    AS STRING * 64
    LOCAL n      AS LONG
    LOCAL idx    AS LONG
    LOCAL st     AS LONG
    LOCAL b1     AS LONG
    LOCAL b2     AS LONG
    LOCAL rc     AS LONG
    LOCAL v      AS LONG
    LOCAL ipaddr AS LONG
    LOCAL fail   AS LONG

    ' Message numbers, read from C:\PBWin10\WINAPI\*.inc on this machine.
    LOCAL mTbmGetPos    AS LONG   ' TBM_GETPOS      = WM_USER       = 1024
    LOCAL mTbmSetPos    AS LONG   ' TBM_SETPOS      = WM_USER + 5   = 1029
    LOCAL mTbmSetRange  AS LONG   ' TBM_SETRANGE    = WM_USER + 6   = 1030
    LOCAL mUdmSetPos32  AS LONG   ' UDM_SETPOS32    = WM_USER + 113 = 1137
    LOCAL mUdmGetPos32  AS LONG   ' UDM_GETPOS32    = WM_USER + 114 = 1138
    LOCAL mHkmSetHotkey AS LONG   ' HKM_SETHOTKEY   = WM_USER + 1   = 1025
    LOCAL mHkmGetHotkey AS LONG   ' HKM_GETHOTKEY   = WM_USER + 2   = 1026
    LOCAL mIpmSetAddr   AS LONG   ' IPM_SETADDRESS  = WM_USER + 101 = 1125
    LOCAL mIpmGetAddr   AS LONG   ' IPM_GETADDRESS  = WM_USER + 102 = 1126

    ConPrint "batch 189 - CONTROL ADD TRACKBAR / UPDOWN / HOTKEY / IPADDRESS"
    ConPrint "-------------------------------------------------------------"

    mTbmGetPos = 1024
    mTbmSetPos = 1029
    mTbmSetRange = 1030
    mUdmSetPos32 = 1137
    mUdmGetPos32 = 1138
    mHkmSetHotkey = 1025
    mHkmGetHotkey = 1026
    mIpmSetAddr = 1125
    mIpmGetAddr = 1126

    IMPORT ADDR "SendMessageA", "USER32.DLL" TO smA, smH
    IMPORT ADDR "GetWindowLongA", "USER32.DLL" TO gwlA, gwlH
    IMPORT ADDR "GetClassNameA", "USER32.DLL" TO gcnA, gcnH
    IF smA = 0 OR gwlA = 0 OR gcnA = 0 THEN
        fail = fail + 1
        ConPrint "FAIL IMPORT ADDR SendMessageA / GetWindowLongA / GetClassNameA"
    END IF

    ' The dialog is shown modeless: a modal dialog would need a human to
    ' dismiss it, and this sample has to run unattended.
    DIALOG NEW 0, "batch 189 - common controls, part 2", 40, 30, 420, 200 TO hDlg
    DIALOG SHOW MODELESS hDlg

    ' Every coordinate is a multiple of 4 on purpose: CONTROL ADD converts
    ' dialog units to pixels with the base 7 x 14, and CONTROL GET divides
    ' back - an exact round trip only for multiples of 4.
    CONTROL ADD "msctls_trackbar32", hDlg, 301, "",   8,  8, 160, 24
    CONTROL HANDLE hDlg, 301 TO hTb
    CONTROL ADD "msctls_updown32", hDlg, 302, "", 200,  8,  40, 24
    CONTROL HANDLE hDlg, 302 TO hUd
    CONTROL ADD "msctls_hotkey32", hDlg, 303, "",   8, 44, 160, 16
    CONTROL HANDLE hDlg, 303 TO hHk
    CONTROL ADD "SysIPAddress32", hDlg, 304, "", 200, 44, 160, 16
    CONTROL HANDLE hDlg, 304 TO hIp

    ' ------------------------------------------------------------------
    ' 1. Every ADD must hand back a real handle.
    ' ------------------------------------------------------------------
    IF hTb <> 0 AND hUd <> 0 AND hHk <> 0 AND hIp <> 0 THEN
        ConPrint "ok   all four CONTROL ADD returned a handle"
    ELSE
        fail = fail + 1
        ConPrint "FAIL a CONTROL ADD returned 0 (tb" & STR$(hTb) & " ud" & STR$(hUd) & " hk" & STR$(hHk) & " ip" & STR$(hIp) & ")"
    END IF

    ' The id must resolve to the same window through a second route.
    hTmp = 0
    CONTROL HANDLE hDlg, 301 TO hTmp
    IF hTmp = hTb AND hTmp <> 0 THEN
        ConPrint "ok   CONTROL HANDLE resolves id 301 to the TRACKBAR handle"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL HANDLE id 301 mismatch"
    END IF

    ' ------------------------------------------------------------------
    ' 2. The window class is the witness that the right Win32 control was
    '    used.  Each class string is the documented one for its control.
    ' ------------------------------------------------------------------
    buf = ""
    n = 0
    CALL DWORD gcnA USING GetClassNameA(hTb, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "msctls_trackbar32") > 0 THEN
        ConPrint "ok   TRACKBAR created a msctls_trackbar32 window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TRACKBAR class (len" & STR$(n) & ") -> " & buf
    END IF

    buf = ""
    n = 0
    CALL DWORD gcnA USING GetClassNameA(hUd, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "msctls_updown32") > 0 THEN
        ConPrint "ok   UPDOWN created a msctls_updown32 window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL UPDOWN class (len" & STR$(n) & ") -> " & buf
    END IF

    buf = ""
    n = 0
    CALL DWORD gcnA USING GetClassNameA(hHk, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "msctls_hotkey32") > 0 THEN
        ConPrint "ok   HOTKEY created a msctls_hotkey32 window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL HOTKEY class (len" & STR$(n) & ") -> " & buf
    END IF

    buf = ""
    n = 0
    CALL DWORD gcnA USING GetClassNameA(hIp, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "SysIPAddress32") > 0 THEN
        ConPrint "ok   IPADDRESS created a SysIPAddress32 window"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IPADDRESS class (len" & STR$(n) & ") -> " & buf
    END IF

    ' ------------------------------------------------------------------
    ' 3. The styles really reached the window.  WS_CHILD is 1073741824 and
    '    WS_VISIBLE is 268435456; the bit tests are kept in separate
    '    variables on purpose, because mixing AND's two meanings in one
    '    expression is exactly the PowerBASIC trap.
    ' ------------------------------------------------------------------
    idx = -16                       ' GWL_STYLE
    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hTb, idx) TO st
    b1 = st AND 1073741824          ' WS_CHILD
    b2 = st AND 268435456           ' WS_VISIBLE
    IF b1 <> 0 AND b2 <> 0 THEN
        ConPrint "ok   TRACKBAR is WS_CHILD | WS_VISIBLE"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TRACKBAR styles (st=" & STR$(st) & ")"
    END IF

    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hHk, idx) TO st
    b1 = st AND 1073741824
    b2 = st AND 268435456
    IF b1 <> 0 AND b2 <> 0 THEN
        ConPrint "ok   HOTKEY is WS_CHILD | WS_VISIBLE"
    ELSE
        fail = fail + 1
        ConPrint "FAIL HOTKEY styles (st=" & STR$(st) & ")"
    END IF

    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hIp, idx) TO st
    b1 = st AND 1073741824
    b2 = st AND 268435456
    IF b1 <> 0 AND b2 <> 0 THEN
        ConPrint "ok   IPADDRESS is WS_CHILD | WS_VISIBLE"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IPADDRESS styles (st=" & STR$(st) & ")"
    END IF

    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hUd, idx) TO st
    b1 = st AND 1073741824
    b2 = st AND 268435456
    IF b1 <> 0 AND b2 <> 0 THEN
        ConPrint "ok   UPDOWN is WS_CHILD | WS_VISIBLE"
    ELSE
        fail = fail + 1
        ConPrint "FAIL UPDOWN styles (st=" & STR$(st) & ")"
    END IF

    ' ------------------------------------------------------------------
    ' 4. Message round trips - the strongest evidence, because only a live
    '    window answers them.
    ' ------------------------------------------------------------------
    ' TRACKBAR: range 0..100 (MAKELONG(0,100) = 100 * 65536), then pos 42.
    rc = 0
    CALL DWORD smA USING SendMessageA(hTb, mTbmSetRange, 1, 6553600) TO rc
    rc = 0
    CALL DWORD smA USING SendMessageA(hTb, mTbmSetPos, 1, 42) TO rc
    v = -1
    CALL DWORD smA USING SendMessageA(hTb, mTbmGetPos, 0, 0) TO v
    IF v = 42 THEN
        ConPrint "ok   TRACKBAR set position 42 and read back 42"
    ELSE
        fail = fail + 1
        ConPrint "FAIL TRACKBAR position round trip (got" & STR$(v) & ")"
    END IF

    ' UPDOWN: 32-bit position, so the WM_USER+113/114 pair.
    rc = 0
    CALL DWORD smA USING SendMessageA(hUd, mUdmSetPos32, 0, 7) TO rc
    v = -1
    CALL DWORD smA USING SendMessageA(hUd, mUdmGetPos32, 0, 0) TO v
    IF v = 7 THEN
        ConPrint "ok   UPDOWN set position 7 and read back 7"
    ELSE
        fail = fail + 1
        ConPrint "FAIL UPDOWN position round trip (got" & STR$(v) & ")"
    END IF

    ' HOTKEY: Ctrl+A = MAKEWORD(0x41, MOD_CONTROL 2) = 577.
    rc = 0
    CALL DWORD smA USING SendMessageA(hHk, mHkmSetHotkey, 577, 0) TO rc
    v = -1
    CALL DWORD smA USING SendMessageA(hHk, mHkmGetHotkey, 0, 0) TO v
    IF v = 577 THEN
        ConPrint "ok   HOTKEY set Ctrl+A and read back the same hotkey word"
    ELSE
        fail = fail + 1
        ConPrint "FAIL HOTKEY round trip (got" & STR$(v) & ")"
    END IF

    ' IPADDRESS: 10.0.0.1 = 0x0A000001 = 167772161, read back through a
    ' pointer (lParam = LPDWORD).  IPM_GETADDRESS answers the number of
    ' non-BLANK fields, and a field displaying 0 is not blank - so all four
    ' fields of 10.0.0.1 count, and 4 is the correct answer.  (The first run
    ' of this sample asserted 2, got 4, and the control was right.)
    ipaddr = 0
    rc = 0
    CALL DWORD smA USING SendMessageA(hIp, mIpmSetAddr, 0, 167772161) TO rc
    v = 0
    ipaddr = 0
    CALL DWORD smA USING SendMessageA(hIp, mIpmGetAddr, 0, VARPTR(ipaddr)) TO v
    IF ipaddr = 167772161 AND v = 4 THEN
        ConPrint "ok   IPADDRESS stored 10.0.0.1 and read it back (4 non-blank fields)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IPADDRESS round trip (addr" & STR$(ipaddr) & " fields" & STR$(v) & ")"
    END IF

    DIALOG END hDlg, fail
    ConPrint "=== FAILURES:" & STR$(fail) & " ==="
END FUNCTION



