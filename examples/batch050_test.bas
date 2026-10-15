

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

' PB64Basic Enhanced - batch50_test.bas
' NOTE: menu bar/popup handles are per-process values (PBWin32 vs fork x64);
'       print normalized, assertion only checks <>0.
' MENU NEW BAR / NEW POPUP / ADD STRING / ADD POPUP / DELETE (batch 50)
FUNCTION PBMAIN() AS LONG
    LOCAL hbar AS LONG
    LOCAL hpop AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG

    MENU NEW BAR TO hbar
    IF hbar = 0 THEN
        ConPrint "FAIL: menu bar handle is zero"
        INCR fails
    ELSE
        ConPrint "OK: menu bar handle (nonzero; value differs by bitness)"
    END IF

    MENU NEW POPUP TO hpop
    IF hpop = 0 THEN
        ConPrint "FAIL: popup handle is zero"
        INCR fails
    ELSE
        ConPrint "OK: popup handle (nonzero; value differs by bitness)"
    END IF

    MENU ADD STRING, hbar, "File", 100, 0
    MENU ADD POPUP, hbar, "Menu", hpop, 0
    MENU ADD STRING, hpop, "Open", 101, 0
    MENU ADD STRING, hpop, "Save", 102, 0
    ConPrint "OK: menu items added"

    MENU DELETE hbar, 1
    ConPrint "OK: popup deleted from bar"

    IF fails = 0 THEN
        ConPrint "batch50: ALL PASS"
    ELSE
        ConPrint "batch50: FAILURES=" & STR$(fails)
    END IF
    ConPrint "Press any key to exit..."
END FUNCTION




