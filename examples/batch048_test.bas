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

' PB64Basic Enhanced - batch48_test.bas
' IMAGELIST NEW / GET COUNT / KILL (tier-3 -> implemented, batch 48)
' NOTE: IMAGELIST NEW's initial& parameter is the INITIAL CAPACITY, not the
' image count (ImageList_Create semantics) — a fresh list has GET COUNT = 0.
FUNCTION PBMAIN() AS LONG
    LOCAL himl AS LONG
    LOCAL cnt AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG

    ' IMAGELIST NEW BITMAP 16, 16, 24, 2 TO himl
    IMAGELIST NEW BITMAP 16, 16, 24, 2 TO himl
    IF himl = 0 THEN
        ConPrint "FAIL: imagelist handle is zero"
        INCR fails
    ELSE
        ConPrint "OK: imagelist handle " & STR$(himl)
    END IF

    ' IMAGELIST GET COUNT himl TO cnt — empty list => 0
    IMAGELIST GET COUNT himl TO cnt
    IF cnt <> 0 THEN
        ConPrint "FAIL: count=" & STR$(cnt) & " expected 0 (empty list)"
        INCR fails
    ELSE
        ConPrint "OK: imagelist count=" & STR$(cnt) & " (empty list)"
    END IF

    ' IMAGELIST KILL himl
    IMAGELIST KILL himl
    ConPrint "OK: imagelist killed"

    IF fails = 0 THEN
        ConPrint "batch48: ALL PASS"
    ELSE
        ConPrint "batch48: FAILURES=" & STR$(fails)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

