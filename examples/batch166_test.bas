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
' PowerBasilisk Enhanced - batch 166 regression test
'---------------------------------------------------------------------
' Purpose:  Two things this batch fixed or added, in one runnable file.
'
' Demonstrates:
'   * CONTROL ADD classname$ - the DDT custom-control statement, in all
'     three operand forms the official page documents.
'   * DIALOG NEW ... TO <LOCAL LONG> delivering a real handle.
'
' Why the second half matters: before batch 166 every DIALOG/CONTROL
' creation arm stored the 64-bit handle with an unconditional i64 store,
' even when the target variable was a 4-byte LOCAL LONG.  The store ran
' past the end of the alloca and the handle read back as 0, so any
' program that declared its dialog handle LOCAL got a null dialog while
' the compiler reported success.  The arms now convert the handle to the
' destination's declared type first, exactly as DIALOG GET SIZE already
' did.  Nine arms were affected.
'
' Expected output (run mode):
'   dialog hwnd   = <nonzero>
'   trackbar hwnd = <nonzero>
'   progress hwnd = <nonzero>
'   statusbar hwnd= <nonzero>
'   PASS
'   (exit code 0)
'
' Complexity note: O(1) - one dialog and three controls, no algorithm.
'
' Notes:  the dialog is opened with DIALOG SHOW MODELESS and closed with
'   DIALOG END, so the sample needs no keyboard input and is safe to run
'   in the non-interactive verification harness.
'   The class names are the documented comctl32 ones - "msctls_trackbar32",
'   "msctls_progress32", "msctls_statusbar32".  Note there is no "CLASS"
'   in these names; "MSCTLS_TRACKBAR_CLASS32" is not a real window class.
'=====================================================================
#COMPILER PBWIN 10

FUNCTION PBMAIN () AS LONG
    LOCAL hDlg    AS LONG      ' deliberately LOCAL - the batch 166 fix
    LOCAL hTrack  AS LONG
    LOCAL hProg   AS LONG
    LOCAL hStatus AS LONG
    LOCAL cls     AS STRING
    LOCAL res     AS LONG

    ' ---- the LOCAL dialog handle must survive DIALOG NEW ---------------
    DIALOG NEW 0, "Batch 166: custom control + local dialog handle", 100, 100, 320, 190 TO hDlg
    IF hDlg = 0 THEN
        ConPrint "FAIL: DIALOG NEW gave a LOCAL handle the value 0"
        FUNCTION = 1
        EXIT FUNCTION
    END IF
    ConPrint "dialog hwnd   =" & STR$(hDlg)
    DIALOG SHOW MODELESS hDlg

    ' ---- 1a. literal class name, no style operands ---------------------
    ' With no style given, the runtime supplies WS_CHILD + WS_VISIBLE so
    ' the control is actually visible; the official custom-control page
    ' warns that DDT applies no default style of its own.
    CONTROL ADD "msctls_trackbar32", hDlg, 101, "", 10, 10, 120, 24
    CONTROL HANDLE hDlg, 101 TO hTrack

    ' ---- 1b. explicit primary style + extended style -------------------
    ' &H50000000 is WS_CHILD + WS_VISIBLE written out by hand.
    CONTROL ADD "msctls_progress32", hDlg, 102, "", 10, 46, 120, 24, &H50000000, 0
    CONTROL HANDLE hDlg, 102 TO hProg

    ' ---- 1c. the class name held in a STRING variable ------------------
    cls = "msctls_statusbar32"
    CONTROL ADD cls, hDlg, 103, "", 10, 82, 290, 24
    CONTROL HANDLE hDlg, 103 TO hStatus

    IF hTrack = 0 OR hProg = 0 OR hStatus = 0 THEN
        ConPrint "FAIL: a custom control came back NULL"
        ConPrint "  trackbar =" & STR$(hTrack) & " progress =" & STR$(hProg) & " statusbar =" & STR$(hStatus)
        DIALOG END hDlg, res
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    ConPrint "trackbar hwnd =" & STR$(hTrack)
    ConPrint "progress hwnd =" & STR$(hProg)
    ConPrint "statusbar hwnd=" & STR$(hStatus)

    DIALOG END hDlg, res
    ConPrint "PASS"
    FUNCTION = 0
END FUNCTION



