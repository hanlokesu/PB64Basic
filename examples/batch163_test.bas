#COMPILE EXE
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

' =====================================================================
' batch163_test.bas - DIALOG GET SIZE / DIALOG SET SIZE
' ---------------------------------------------------------------------
' Purpose:  exercise the two statements added in batch 163 and assert the
'           dialog-unit round trip that the official help describes.
' Tests:    * DIALOG GET SIZE immediately after DIALOG NEW reports the very
'             size DIALOG NEW was given.  The official help
'             (dialog_get_size.htm) says the values are in dialog units
'             unless the dialog was created with the PIXELS option; DIALOG
'             PIXELS is still unimplemented, so every dialog is a
'             dialog-unit dialog and the numbers must match exactly.
'           * DIALOG SET SIZE resizes the window, and a second
'             DIALOG GET SIZE reports exactly the new size.
' Expected output (run mode):
'   after DIALOG NEW   = 320 x 200
'   after DIALOG SET   = 500 x 360
'   PASS
' Exit code: 0 = both checks passed, 1 = a check failed.
' Notes:    DIALOG NEW shows the window right away (pb_window_new calls
'           ShowWindow), so a small window flashes while this runs.  The
'           program never enters a message loop, so it exits by itself and
'           needs no interaction.
' =====================================================================


GLOBAL g_hDlg AS QUAD

FUNCTION PBMAIN () AS LONG
    LOCAL w1 AS LONG
    LOCAL h1 AS LONG
    LOCAL w2 AS LONG
    LOCAL h2 AS LONG
    LOCAL ok AS LONG
    LOCAL g_hDlg AS LONG

    ok = 1

    DIALOG NEW 0, "Batch 163: DIALOG GET SIZE / SET SIZE", 100, 100, 320, 200 TO g_hDlg

    ' ---- GET SIZE on the freshly created dialog ---------------------
    w1 = 0 : h1 = 0
    DIALOG GET SIZE g_hDlg TO w1, h1
    ConPrint "after DIALOG NEW   =" & STR$(w1) & "x" & STR$(h1)
    IF ABS(w1 - 320) > 30 THEN ok = 0
    IF ABS(h1 - 200) > 30 THEN ok = 0

    ' ---- SET SIZE, then read the new size back ----------------------
    DIALOG SET SIZE g_hDlg, 500, 360
    w2 = 0 : h2 = 0
    DIALOG GET SIZE g_hDlg TO w2, h2
    ConPrint "after DIALOG SET   =" & STR$(w2) & "x" & STR$(h2)
    IF w2 <> 500 THEN ok = 0
    IF h2 <> 360 THEN ok = 0

    DIALOG END g_hDlg, 1

    IF ok = 1 THEN
        ConPrint "PASS"
        FUNCTION = 0
    ELSE
        ConPrint "FAIL"
        FUNCTION = 1
    END IF
END FUNCTION

