

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
' batch162_test.bas - batch 162: the docs accuracy pass
'---------------------------------------------------------------------
' statement-coverage.md still claimed two unimplemented keywords (METRICS and
' UCODE$).  Both had in fact been implemented by batch 160 - batch 162 corrected
' the stale line.  This sample is the witness for that claim: the three
' functions the corrected line talks about are called here and their results are
' asserted, so a regression re-opens the question with a failing run rather than
' with a sentence in a document.
'
' Asserts:
'   * UCODE$("ABC") has 6 bytes  (the documented "doubles the byte count")
'   * ACODE$(UCODE$("ABC")) round-trips back to "ABC"
'   * METRICS(0) (SM_CXSCREEN) is positive
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL fail AS LONG
    LOCAL n AS LONG
    LOCAL u AS STRING
    LOCAL back AS STRING
    n = METRICS(Scroll.Horz)
    u = UCODE$("ABC")
    back = ACODE$(u)
    ConPrint "METRICS(ScrollHorz) = " & STR$(n)
    ConPrint "UCODE$ len   = " & STR$(LEN(u))
    ConPrint "ACODE$ back  = " & back
    IF n <= 0 THEN
        fail = fail + 1
    END IF
    IF LEN(u) <> 6 THEN
        fail = fail + 1
    END IF
    IF back <> "ABC" THEN
        fail = fail + 1
    END IF
    ConPrint "=== FAILURES:" & STR$(fail) & "==="
    FUNCTION = fail
' Press any key to exit...
ConWaitKey
END FUNCTION





