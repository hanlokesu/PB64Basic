' === console emulation for dual-compiler compatibility ===
' (PBWin10 has no PRINT/#CONSOLE; this wrapper uses only official Win32 API)
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpNumberOfBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
DECLARE FUNCTION ReadFile LIB "KERNEL32.DLL" ALIAS "ReadFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToRead AS DWORD, lpNumberOfBytesRead AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
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
' batch 224 - dual-compiler example conversion witness
'---------------------------------------------------------------------
' The ConPrint/ConWaitKey wrapper above compiles and runs identically on
' PowerBASIC 10 (PBWin10) and on this fork.  This witness prints two
' lines through ConPrint and waits for a key with ConWaitKey - the
' pattern every console example in examples/ now follows.
' Expected output (headless): B224-CONPRINT-1 / B224-CONPRINT-2
' Expected output (interactive): the two lines, then "waiting for any
'   key to exit..." and a keypress returns to the prompt.
'=====================================================================
FUNCTION PBMAIN() AS LONG
    ConPrint "B224-CONPRINT-1"
    ConPrint "B224-CONPRINT-2"
    ConPrint "waiting for any key to exit..."
    ConWaitKey
    FUNCTION = 0
END FUNCTION

