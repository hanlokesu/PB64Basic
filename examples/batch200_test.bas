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
' PB64Basic - batch 200 test: the 64-bit handle must reach the variable
'---------------------------------------------------------------------
' Purpose:   pin the batch 200e defect and its fix.  Before the fix, a handle assigned to a
'            variable never arrived: the generated code stored 8 bytes into a 4-byte slot, the
'            backend saw an out-of-bounds store and dropped it, so the variable stayed 0.
' Note:      an ImageList handle is pointer-sized (64-bit here), so the variable that holds it
'            must be QUAD - a LONG truncates it and later calls get an invalid handle.
' Asserts:   1. the handle is non-zero after IMAGELIST NEW
'            2. the handle still works: GET COUNT on it, then KILL
' Expected:  every printed value non-zero; count of a fresh list is 0 (correct).
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL waitk AS STRING
    LOCAL hil AS LONG
    LOCAL cnt AS LONG
    IMAGELIST NEW BITMAP 16, 16, 32, 4 TO hil
    ConPrint "QUAD handle  = " & STR$(hil)
    IF hil = 0 THEN
        ConPrint "FAIL: handle lost (batch 200e defect)"
        FUNCTION = 1
        EXIT FUNCTION
    END IF
    IMAGELIST GET COUNT hil TO cnt
    ConPrint "GET COUNT    = " & STR$(cnt)
    ConPrint "PASS: handle survived and the list is usable"
    IMAGELIST KILL hil
    FUNCTION = 0
' Press any key to exit...
    waitk = WAITKEY$
END FUNCTION

