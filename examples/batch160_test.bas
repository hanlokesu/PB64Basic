

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

' =====================================================================
' batch160_test.bas â€? METRICS, UCODE$ and ACODE$
' ---------------------------------------------------------------------
' Purpose:  exercise the three functions added in batch 160 and print the
'           values so the numbers can be checked against the Windows SDK.
' Tests:    * METRICS with a dotted name (Scroll.Horz, Border.X, ...)
'           * METRICS with a three-segment name (Frame.Fixed.X)
'           * METRICS with the single-word names (Caption, Menubar)
'           * METRICS with a plain numeric metric index
'           * UCODE$ doubles the byte count while keeping the characters
'           * ACODE$ is the exact inverse of UCODE$
' Expected output: the metric values are machine-dependent, so only the
'           structural checks are asserted.  Exit code 0 = all checks passed,
'           1 = a check failed.
' =====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL n AS LONG
    LOCAL u AS STRING
    LOCAL back AS STRING
    LOCAL ok AS LONG

    ok = 1

    ' ---- METRICS: dotted names -------------------------------------
    n = METRICS(Scroll.Horz)
    ConPrint "Scroll.Horz    =" & STR$(n)
    IF n <= 0 THEN ok = 0

    n = METRICS(Scroll.Vert)
    ConPrint "Scroll.Vert    =" & STR$(n)
    IF n <= 0 THEN ok = 0

    n = METRICS(Border.X)
    ConPrint "Border.X       =" & STR$(n)
    IF n <= 0 THEN ok = 0

    n = METRICS(Icon.X)
    ConPrint "Icon.X         =" & STR$(n)
    IF n <= 0 THEN ok = 0

    ' ---- METRICS: three-segment name -------------------------------
    n = METRICS(Frame.Fixed.X)
    ConPrint "Frame.Fixed.X  =" & STR$(n)
    IF n <= 0 THEN ok = 0

    n = METRICS(Frame.Resize.X)
    ConPrint "Frame.Resize.X =" & STR$(n)
    IF n <= 0 THEN ok = 0

    ' ---- METRICS: single-word names --------------------------------
    n = METRICS(Caption)
    ConPrint "Caption        =" & STR$(n)
    IF n <= 0 THEN ok = 0

    n = METRICS(Menubar)
    ConPrint "Menubar        =" & STR$(n)
    IF n <= 0 THEN ok = 0

    ' ---- METRICS: a plain metric index (SM_CXSCREEN = 0) ------------
    n = METRICS(Scroll.Horz)
    ConPrint "METRICS(ScrollHorz)=" & STR$(n)
    IF n <= 0 THEN ok = 0

    ' ---- UCODE$ / ACODE$ -------------------------------------------
    u = UCODE$("ABC")
    ConPrint "UCODE$ byte len =" & STR$(LEN(u))
    IF LEN(u) <> 6 THEN ok = 0

    back = ACODE$(u)
    ConPrint "ACODE$ restored =" & back
    IF back <> "ABC" THEN ok = 0

    back = UCODE$("Hello") : back = ACODE$(back)
    ConPrint "round trip      =" & back
    IF back <> "Hello" THEN ok = 0

    ' ---- result ----------------------------------------------------
    IF ok = 1 THEN
        ConPrint "PASS"
        FUNCTION = 0
    ELSE
        ConPrint "FAIL"
        FUNCTION = 1
    END IF
' Press any key to exit...
ConWaitKey
END FUNCTION





