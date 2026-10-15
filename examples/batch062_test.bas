

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

' PB64Basic Enhanced - batch62_test.bas
' MENU GET STATE / SET STATE / GET TEXT / SET TEXT (batch 62)
FUNCTION PBMAIN() AS LONG
    LOCAL hbar AS LONG
    LOCAL hpop AS LONG
    LOCAL st AS LONG
    LOCAL txt AS STRING
    LOCAL ok AS LONG
    LOCAL waitk AS STRING

    MENU NEW BAR TO hbar
    MENU NEW POPUP TO hpop
    MENU ADD STRING, hbar, "File", 100, 0
    MENU ADD STRING, hbar, "Edit", 101, 0
    MENU ADD STRING, hpop, "Open", 110, 0
    MENU ADD STRING, hpop, "Save", 111, 0

    ' 1. GET STATE by position (enabled = 0)
    MENU GET STATE hbar, 1 TO st
    IF st = 0 THEN
        ok = ok + 1
    ELSE
        ConPrint "GET STATE POS FAIL" & STR$(st)
    END IF

    ' 2. GET STATE by command id
    MENU GET STATE hbar, BYCMD 100 TO st
    IF st = 0 THEN
        ok = ok + 1
    ELSE
        ConPrint "GET STATE BYCMD FAIL" & STR$(st)
    END IF

    ' 3. SET STATE grayed (1) by command id, then read back
    MENU SET STATE hbar, BYCMD 100, 1
    MENU GET STATE hbar, BYCMD 100 TO st
    IF st AND 1 THEN
        ok = ok + 1
    ELSE
        ConPrint "SET GRAYED FAIL" & STR$(st)
    END IF

    ' 4. SET STATE enabled (0)
    MENU SET STATE hbar, BYCMD 100, 0
    MENU GET STATE hbar, BYCMD 100 TO st
    IF (st AND 1) = 0 THEN
        ok = ok + 1
    ELSE
        ConPrint "SET ENABLED FAIL" & STR$(st)
    END IF

    ' 5. SET STATE checked (8) on popup item, read back
    MENU SET STATE hpop, BYCMD 110, 8
    MENU GET STATE hpop, BYCMD 110 TO st
    IF st AND 8 THEN
        ok = ok + 1
    ELSE
        ConPrint "SET CHECKED FAIL" & STR$(st)
    END IF

    ' 6. GET TEXT by command id
    MENU GET TEXT hbar, BYCMD 100 TO txt
    IF txt = "File" THEN
        ok = ok + 1
    ELSE
        ConPrint "GET TEXT FAIL [" & txt & "]"
    END IF

    ' 7. SET TEXT by command id, then read back
    MENU SET TEXT hbar, BYCMD 100, "Filer"
    MENU GET TEXT hbar, BYCMD 100 TO txt
    IF txt = "Filer" THEN
        ok = ok + 1
    ELSE
        ConPrint "SET TEXT FAIL [" & txt & "]"
    END IF

    ' 8. GET TEXT by position on popup
    MENU GET TEXT hpop, 2 TO txt
    IF txt = "Save" THEN
        ok = ok + 1
    ELSE
        ConPrint "GET TEXT POS FAIL [" & txt & "]"
    END IF

    IF ok = 8 THEN
        ConPrint "batch62: ALL PASS"
    ELSE
        ConPrint "batch62: FAILURES=" & STR$(8 - ok)
    END IF
    ConPrint "Press any key to exit..."
END FUNCTION




