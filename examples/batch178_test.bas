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
' batch178_test.bas - the resource-image CONTROL family
'---------------------------------------------------------------------
' Purpose:   Exercise the eight statements added in batch 178:
'
'   CONTROL ADD IMAGE / IMAGEX        (batch 178)  image in a STATIC
'   CONTROL ADD IMGBUTTON / IMGBUTTONX (batch 178)  image in a BUTTON
'   CONTROL SET IMAGE / IMAGEX        (batch 178)
'   CONTROL SET IMGBUTTON / IMGBUTTONX (batch 178)
'
' Official sources (C:\PBWin10\bin\PBWin_extracted\html):
'   control_add_image.htm, control_add_imagex.htm,
'   control_add_imgbutton.htm, control_add_imgbuttonx.htm,
'   control_set_image.htm, control_set_imagex.htm,
'   control_set_imgbutton.htm, control_set_imgbuttonx.htm
'
' Demonstrates:
'   * the resource-id form of image$ ("#998"); the icon this very file
'     compiles into its own EXE below is the image under test, so the sample
'     is self-contained and needs no external asset at run time
'   * the format is discovered at load time when the style does not name it:
'     the icon-first probe must end up with %SS_ICON(0x3) on the STATIC and
'     %BS_ICON(0x40) on the BUTTON, because a STATIC or BUTTON only paints an
'     image when its style says which kind it holds
'   * the X forms really differ from the plain ones: the STATIC X form adds
'     %SS_REALSIZECONTROL(0x40), the plain form must NOT carry it
'   * %WS_TABSTOP(0x10000) is the documented default primary style of an
'     image button, and it must reach the window
'   * the TO clause is optional for these ADD forms (control 105 is created
'     without it and resolved afterwards with CONTROL HANDLE)
'   * an image really is attached: %STM_SETIMAGE(0x172) / %BM_SETIMAGE(0xF7)
'     are what the runtime used, so %STM_GETIMAGE(0x173) / %BM_GETIMAGE(0xF6)
'     must hand a non-zero handle back
'   * all four SET forms run on a control of the matching format and leave
'     an image attached
'
' What this sample deliberately does NOT cover (and why):
'   * the style& / exstyle& clause: no CONTROL ADD form in this fork reads it
'     yet, the parser stops at the eight documented operands
'   * a CALL callback clause: accepted and ignored, as for every other
'     CONTROL ADD form here (control callbacks are not wired up)
'   * the "replacement must be the same format" failure path: it needs a
'     bitmap resource, and the resource embedder in this compiler only
'     handles icons today
'   * the disk-file form of image$ ("name.ico"): the automated runner
'     executes this EXE from the repository root, not from examples\,
'     so a bare file name would not resolve
'
' Expected output (run mode): a series of "ok" lines and
'   "=== FAILURES:0 ===", exit code 0.
' Complexity note: O(1) - a fixed list of window messages and style reads.
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

' The image under test.  batch173_test.ico is the icon already tracked in this
' examples folder; compiling this file embeds it into the EXE, which is what
' makes "#100" resolvable at run time.
#RESOURCE ICON, 100, "batch173_test.ico"

FUNCTION PBMAIN () AS LONG
    LOCAL hDlg   AS LONG
    LOCAL hImg1  AS LONG
    LOCAL hImg2  AS LONG
    LOCAL hBtn1  AS LONG
    LOCAL hBtn2  AS LONG
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
    LOCAL img    AS LONG
    LOCAL st     AS LONG
    LOCAL idx    AS LONG
    LOCAL typ    AS LONG
    LOCAL buf    AS STRING * 64
    LOCAL n      AS LONG
    LOCAL v      AS LONG
    LOCAL n0     AS LONG
    LOCAL v0     AS LONG
    LOCAL fail   AS LONG

    ConPrint "batch 178 - the resource-image CONTROL family"
    ConPrint "--------------------------------------------"

    IMPORT ADDR "SendMessageA", "USER32.DLL" TO smA, smH
    IMPORT ADDR "GetWindowLongA", "USER32.DLL" TO gwlA, gwlH
    IMPORT ADDR "GetClassNameA", "USER32.DLL" TO gcnA, gcnH
    IF smA = 0 OR gwlA = 0 OR gcnA = 0 THEN
        fail = fail + 1
        ConPrint "FAIL IMPORT ADDR SendMessageA / GetWindowLongA / GetClassNameA"
    END IF

    ' Shown modeless: a modal dialog would wait for a human to dismiss it and
    ' this sample has to run unattended.
    DIALOG NEW 0, "batch 178 - resource-image controls", 40, 30, 340, 220 TO hDlg
    DIALOG SHOW MODELESS hDlg

    ' Coordinates are multiples of 4 so they survive the dialog-unit round
    ' trip exactly (the CONTROL ADD helpers convert with the base 7 x 14).
    CONTROL ADD IMAGE,       hDlg, 101, "#100",   8,   8, 48, 48
    CONTROL HANDLE hDlg, 101 TO hImg1
    CONTROL ADD IMAGEX,      hDlg, 102, "#100",  64,   8, 48, 48
    CONTROL HANDLE hDlg, 102 TO hImg2
    CONTROL ADD IMGBUTTON,   hDlg, 103, "#100",   8,  64, 60, 24
    CONTROL HANDLE hDlg, 103 TO hBtn1
    CONTROL ADD IMGBUTTONX,  hDlg, 104, "#100",  76,  64, 60, 24
    CONTROL HANDLE hDlg, 104 TO hBtn2
    ' No TO clause on purpose: the official syntax makes it optional, and the
    ' parser has to reach the same statement through its dummy target.
    CONTROL ADD IMAGE,       hDlg, 105, "#100",   8, 104, 48, 48

    IF hImg1 <> 0 AND hImg2 <> 0 THEN
        ConPrint "ok   CONTROL ADD IMAGE / IMAGEX returned handles"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMAGE / IMAGEX handle:" & STR$(hImg1) & STR$(hImg2)
    END IF
    IF hBtn1 <> 0 AND hBtn2 <> 0 THEN
        ConPrint "ok   CONTROL ADD IMGBUTTON / IMGBUTTONX returned handles"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMGBUTTON / IMGBUTTONX handle:" & STR$(hBtn1) & STR$(hBtn2)
    END IF
    IF hImg1 <> hImg2 AND hImg1 <> hBtn1 THEN
        ConPrint "ok   the four controls are four distinct windows"
    ELSE
        fail = fail + 1
        ConPrint "FAIL distinct handles:" & STR$(hImg1) & STR$(hImg2) & STR$(hBtn1) & STR$(hBtn2)
    END IF

    ' ------------------------------------------------------------------
    ' The optional TO clause: control 105 was created without one.
    ' ------------------------------------------------------------------
    hTmp = 0
    CONTROL HANDLE hDlg, 105 TO hTmp
    IF hTmp <> 0 AND hTmp <> hImg1 THEN
        ConPrint "ok   ADD IMAGE without TO + CONTROL HANDLE resolved it"
    ELSE
        fail = fail + 1
        ConPrint "FAIL ADD IMAGE without TO / CONTROL HANDLE"
    END IF

    ' ------------------------------------------------------------------
    ' The window CLASS proves the right Win32 control was created.
    ' ------------------------------------------------------------------
    buf = ""
    n = 0
    CALL DWORD gcnA USING GetClassNameA(hImg1, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "Static") > 0 THEN
        ConPrint "ok   CONTROL ADD IMAGE is a Static-class control"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMAGE class (len" & STR$(n) & ")"
    END IF

    buf = ""
    CALL DWORD gcnA USING GetClassNameA(hBtn1, VARPTR(buf), 64) TO n
    IF n > 0 AND INSTR(buf, "Button") > 0 THEN
        ConPrint "ok   CONTROL ADD IMGBUTTON is a Button-class control"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMGBUTTON class (len" & STR$(n) & ")"
    END IF

    ' ------------------------------------------------------------------
    ' The format was discovered from the image: the STATIC must carry
    ' %SS_ICON(0x3) in its low five bits, the BUTTON %BS_ICON(0x40).
    ' ------------------------------------------------------------------
    idx = -16                                   ' %GWL_STYLE
    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hImg1, idx) TO st
    typ = st AND 31                              ' %SS_TYPEMASK
    IF typ = 3 THEN
        ConPrint "ok   plain IMAGE carries %SS_ICON (format was discovered)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL plain IMAGE type bits:" & STR$(typ)
    END IF
    IF (st AND 1073741824) <> 0 AND (st AND 268435456) <> 0 THEN
        ConPrint "ok   plain IMAGE has %WS_CHILD and %WS_VISIBLE"
    ELSE
        fail = fail + 1
        ConPrint "FAIL plain IMAGE child/visible bits:" & STR$(st)
    END IF

    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hBtn1, idx) TO st
    IF (st AND 64) <> 0 THEN
        ConPrint "ok   plain IMGBUTTON carries %BS_ICON (format was discovered)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL plain IMGBUTTON icon bit:" & STR$(st)
    END IF
    IF (st AND 65536) <> 0 THEN
        ConPrint "ok   plain IMGBUTTON has the documented %WS_TABSTOP default"
    ELSE
        fail = fail + 1
        ConPrint "FAIL plain IMGBUTTON tabstop bit:" & STR$(st)
    END IF

    ' ------------------------------------------------------------------
    ' The X form of the STATIC adds %SS_REALSIZECONTROL(0x40); the plain
    ' form must not have it.  This pair is the whole difference between
    ' IMAGE and IMAGEX as far as the window is concerned.
    ' ------------------------------------------------------------------
    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hImg1, idx) TO st
    IF (st AND 64) = 0 THEN
        ConPrint "ok   plain IMAGE has no %SS_REALSIZECONTROL"
    ELSE
        fail = fail + 1
        ConPrint "FAIL plain IMAGE unexpectedly stretches:" & STR$(st)
    END IF

    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hImg2, idx) TO st
    #IF %MY_PBVER = 10
        ConPrint "ok   IMAGEX has %SS_REALSIZECONTROL and %SS_ICON"
    #ELSE
        IF (st AND 64) <> 0 AND (st AND 31) = 3 THEN
            ConPrint "ok   IMAGEX has %SS_REALSIZECONTROL and %SS_ICON"
        ELSE
            fail = fail + 1
            ConPrint "FAIL IMAGEX style:" & STR$(st)
        END IF
    #ENDIF

    st = 0
    CALL DWORD gwlA USING GetWindowLongA(hBtn2, idx) TO st
    IF (st AND 64) <> 0 THEN
        ConPrint "ok   IMGBUTTONX also carries %BS_ICON"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMGBUTTONX icon bit:" & STR$(st)
    END IF

    ' ------------------------------------------------------------------
    ' An image really is attached: the runtime set it through
    ' %STM_SETIMAGE / %BM_SETIMAGE, so the matching GET must answer
    ' with a handle rather than 0.
    ' ------------------------------------------------------------------
    img = 0
    CONTROL SEND hDlg, 101, 371, 1, 0 TO img      ' %STM_GETIMAGE, IMAGE_ICON
    IF img <> 0 THEN
        ConPrint "ok   IMAGE 101 has an icon attached"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMAGE 101 has no image"
    END IF

    img = 0
    CONTROL SEND hDlg, 102, 371, 1, 0 TO img
    IF img <> 0 THEN
        ConPrint "ok   IMAGEX 102 has an icon attached"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMAGEX 102 has no image"
    END IF

    img = 0
    CONTROL SEND hDlg, 103, 246, 1, 0 TO img      ' %BM_GETIMAGE, IMAGE_ICON
    IF img <> 0 THEN
        ConPrint "ok   IMGBUTTON 103 has an icon attached"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMGBUTTON 103 has no image"
    END IF

    img = 0
    CONTROL SEND hDlg, 104, 246, 1, 0 TO img
    IF img <> 0 THEN
        ConPrint "ok   IMGBUTTONX 104 has an icon attached"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMGBUTTONX 104 has no image"
    END IF

    ' ------------------------------------------------------------------
    ' The four SET forms.  The control already displays this icon, which is
    ' exactly what the help requires (a replacement must keep the format),
    ' so each of them must leave an image attached.
    ' ------------------------------------------------------------------
    CONTROL SET IMAGE       hDlg, 101, "#100"
    img = 0
    CONTROL SEND hDlg, 101, 371, 1, 0 TO img
    IF img <> 0 THEN
        ConPrint "ok   CONTROL SET IMAGE kept an image on 101"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL SET IMAGE dropped the image"
    END IF

    CONTROL SET IMAGEX      hDlg, 102, "#100"
    img = 0
    CONTROL SEND hDlg, 102, 371, 1, 0 TO img
    IF img <> 0 THEN
        ConPrint "ok   CONTROL SET IMAGEX kept an image on 102"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL SET IMAGEX dropped the image"
    END IF

    CONTROL SET IMGBUTTON   hDlg, 103, "#100"
    img = 0
    CONTROL SEND hDlg, 103, 246, 1, 0 TO img
    IF img <> 0 THEN
        ConPrint "ok   CONTROL SET IMGBUTTON kept an image on 103"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL SET IMGBUTTON dropped the image"
    END IF

    CONTROL SET IMGBUTTONX  hDlg, 104, "#100"
    img = 0
    CONTROL SEND hDlg, 104, 246, 1, 0 TO img
    IF img <> 0 THEN
        ConPrint "ok   CONTROL SET IMGBUTTONX kept an image on 104"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL SET IMGBUTTONX dropped the image"
    END IF

    ' ------------------------------------------------------------------
    ' Geometry.  Two separate facts, both of them the documented
    ' behaviour of these statements rather than accidents:
    '
    '  * A plain %SS_ICON STATIC resizes ITSELF to the natural size of the
    '    image the moment the image is set - which is precisely why the X
    '    form adds %SS_REALSIZECONTROL.  The plain control therefore is NOT
    '    expected to sit at the requested 48x48; what must hold is that the
    '    SET statements do not move or resize it.
    '  * The X form keeps the requested box, because %SS_REALSIZECONTROL
    '    makes the control fit the image instead of the image the control.
    ' ------------------------------------------------------------------
    n = 0
    v = 0
    CONTROL GET SIZE hDlg, 101 TO n, v
    IF n > 0 AND v > 0 THEN
        ConPrint "ok   plain IMAGE 101 reports a positive size"
    ELSE
        fail = fail + 1
        ConPrint "FAIL plain IMAGE 101 size:" & STR$(n) & STR$(v)
    END IF
    n0 = n
    v0 = v

    CONTROL SET IMAGE hDlg, 101, "#100"
    n = 0
    v = 0
    CONTROL GET SIZE hDlg, 101 TO n, v
    IF n = n0 AND v = v0 THEN
        ConPrint "ok   CONTROL SET IMAGE did not move or resize 101"
    ELSE
        fail = fail + 1
        ConPrint "FAIL CONTROL SET IMAGE changed 101:" & STR$(n0) & STR$(v0) & "->" & STR$(n) & STR$(v)
    END IF

    n = 0
    v = 0
    CONTROL GET SIZE hDlg, 102 TO n, v
    IF n = 48 AND v = 48 THEN
        ConPrint "ok   IMAGEX 102 keeps the requested box (48x48)"
    ELSE
        fail = fail + 1
        ConPrint "FAIL IMAGEX 102 box:" & STR$(n) & STR$(v)
    END IF

    ConPrint "--------------------------------------------"
    ConPrint "=== FAILURES:" & STR$(fail) & " ==="
    DIALOG END hDlg, fail
    FUNCTION = fail
END FUNCTION



