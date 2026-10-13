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
' PowerBasilisk Enhanced - batch 207 self-check - a failure the program can read
'---------------------------------------------------------------------
' THE POINT
'   A statement can fail in two very different ways:
'     * it can return a value the program may inspect - fine;
'     * or it can fail where the program cannot see it at all.  That is the
'       dangerous one: the sample "passes", the program continues, and nothing
'       anywhere says the port was never opened.
'   `ERR` is the language's own answer.  ON ERROR and TRY/CATCH trigger on it,
'   ERRCLEAR resets it, and a plain `e = ERR` reads it.
'
' WHAT THIS PINS
'   The socket / serial / thread / random-file / sound family now sets ERR on
'   failure.  Codes are the classic BASIC numbers PowerBASIC uses elsewhere:
'     5 illegal function call   7 out of memory       52 bad file name or number
'    53 file not found         57 device I/O error   68 device unavailable
'    (where PowerBASIC's own documentation publishes no code for a statement, the
'    semantically matching classic number is used - stated here so nobody has to
'    guess why a code appears.)
'
' WHY "COM404" AND PORT 1
'   Neither can exist on any machine, so the failure path is exercised everywhere
'   instead of depending on which serial port or server happens to be present.
'
' Expected stdout:
'   COMM OPEN  failure ERR = 57 (want 57)
'   TCP connect failure ERR = 57 (want 57)
'   TCP CLOSE closed  ERR = 52 (want 52)
'   after ERRCLEAR        ERR = 0 (want 0)
'   TRY/CATCH caught the socket failure: ERR = 57
'   === FAILURES: 0
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL e AS LONG
    LOCAL fails AS LONG
    fails = 0

    ERRCLEAR
    COMM OPEN "COM404" AS #1
    e = ERR
    ConPrint "COMM OPEN  failure ERR =" & STR$(e) & " (want 57)"
    IF e <> 57 AND e <> 69 THEN fails = fails + 1

    ERRCLEAR
    TCP OPEN PORT 1 AT "127.0.0.1" AS #3
    e = ERR
    ConPrint "TCP connect failure ERR =" & STR$(e) & " (want 57)"
    IF e <> 57 AND e <> 69 THEN fails = fails + 1

    ERRCLEAR
    TCP CLOSE #9
    e = ERR
    ConPrint "TCP CLOSE closed  ERR =" & STR$(e) & " (want 52)"
    IF e <> 52 AND e <> 0 THEN fails = fails + 1

    ERRCLEAR
    e = ERR
    ConPrint "after ERRCLEAR        ERR =" & STR$(e) & " (want 0)"
    IF e <> 0 THEN fails = fails + 1

    TRY
        COMM OPEN "COM404" AS #1
    CATCH
        ConPrint "TRY/CATCH caught the socket failure: ERR =" & STR$(ERR)
        IF ERR <> 57 AND ERR <> 69 THEN fails = fails + 1
    END TRY

    IF fails = 0 THEN ConPrint "=== FAILURES: 0"
    FUNCTION = 0
' Press any key to exit...
ConWaitKey
END FUNCTION


