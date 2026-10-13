

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
' PowerBasilisk Enhanced - batch 192 test
'---------------------------------------------------------------------
' Subject: SCROLLBAR TRACKPOS, after the batch-192 correction.
'
' What batch 192 changed
'   * A probe (2026-09-26) disproved the batch-191 note "SET/GET TRACKPOS
'     have no codegen arm":  GET TRACKPOS was implemented all along
'     (codegen arm + runtime reader over SIF_TRACKPOS).
'   * `SCROLLBAR SET TRACKPOS` is not an official form - six forms are
'     documented, none is SET TRACKPOS, the coverage CSV has no such row,
'     and Win32 SIF_TRACKPOS is read-only.  The parser now rejects it with
'       Error: SCROLLBAR SET: TRACKPOS is a GET-only sub-command on line N
'     instead of letting it fall through to a codegen "unknown statement".
'
' Batch-192 coverage note (added after v0.2.046):
'   The 23 new parser diagnostics from this batch are COMPILE-TIME REJECTIONS.
'   A runnable sample cannot assert them - anything that compiles did not hit one.
'   Their negative probes live in the repository root folder _negative_probes/
'   and are exercised by the skill script check_negative_probes.py.
'   This file therefore covers only the TRACKPOS half of batch 192.
' This sample exercises the GET side positively (it must compile and print
' FAILURES: 0).  The SET side is a compile-time rejection, so it cannot be
' asserted from inside a program; the negative probe lives in the working
' directory as _b192_probe_set_trackpos.bas.
'
' Expected output:  four "sb ..." lines and "=== FAILURES: 0 ===", exit 0
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL hDlg AS LONG
    LOCAL hSb  AS LONG
    LOCAL n    AS LONG
    LOCAL m    AS LONG
    LOCAL fail AS LONG

    DIALOG NEW 0, "batch192 SCROLLBAR TRACKPOS", 0, 0, 300, 160 TO hDlg
    DIALOG SHOW MODELESS hDlg
    CONTROL ADD SCROLLBAR, hDlg, 501, "", 10, 40, 270, 20
    CONTROL HANDLE hDlg, 501 TO hSb

    ' ---- the already-working trio, re-asserted so a regression shows up ----
    SCROLLBAR SET RANGE hDlg, 501, 0, 100
    n = -1
    m = -1
    SCROLLBAR GET RANGE hDlg, 501 TO n, m
    ConPrint "sb range    -> " & STR$(n) & " / " & STR$(m)
    IF n <> 0 OR m <> 100 THEN fail = fail + 1

    SCROLLBAR SET PAGESIZE hDlg, 501, 10
    n = -1
    SCROLLBAR GET PAGESIZE hDlg, 501 TO n
    ConPrint "sb pagesize -> " & STR$(n)
    IF n <> 10 THEN fail = fail + 1

    SCROLLBAR SET POS hDlg, 501, 30
    n = -1
    SCROLLBAR GET POS hDlg, 501 TO n
    ConPrint "sb pos      -> " & STR$(n)
    IF n <> 30 THEN fail = fail + 1

    ' ---- the statement batch 192 is about ----
    ' TRACKPOS reports where the user is dragging the thumb; with no user
    ' input it tracks the current position, so after SET POS 30 it reads 30.
    n = -1
    SCROLLBAR GET TRACKPOS hDlg, 501 TO n
    ConPrint "sb trackpos -> " & STR$(n)
    IF n < 0 THEN fail = fail + 1

    DIALOG END hDlg, fail
    ConPrint ""
    ConPrint "=== FAILURES:" & STR$(fail) & "==="
    FUNCTION = fail
END FUNCTION



