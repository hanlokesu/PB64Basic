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

' PB64Basic Enhanced - batch47_test.bas
' NOTE: font handles are per-process values (PBWin32 vs fork x64); print normalized,
'       assertion only checks <>0. See dual-run normalization recipe.
' FONT NEW / FONT END (tier-3 -> implemented, batch 47)
FUNCTION PBMAIN() AS LONG
    LOCAL hfont AS LONG
    LOCAL h2 AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG

    ' FONT NEW "Arial", 12 TO hfont
    FONT NEW "Arial", 12 TO hfont
    IF hfont = 0 THEN
        ConPrint "FAIL: font handle is zero"
        INCR fails
    ELSE
        ConPrint "OK: font handle (nonzero; value differs by bitness)"
    END IF

    ' FONT NEW with style (bold+italic), points, charset
    FONT NEW "Courier New", 10, 3, 0, 0, 0 TO h2
    IF h2 = 0 THEN
        ConPrint "FAIL: styled font handle is zero"
        INCR fails
    ELSE
        ConPrint "OK: styled font handle (nonzero; value differs by bitness)"
    END IF

    ' FONT END both
    FONT END hfont
    FONT END h2
    ConPrint "OK: fonts ended"

    IF fails = 0 THEN
        ConPrint "batch47: ALL PASS"
    ELSE
        ConPrint "batch47: FAILURES=" & STR$(fails)
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


