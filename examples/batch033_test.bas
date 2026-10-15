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

' PB64Basic Enhanced - batch 33: CALLSTK call-stack tracing
' Tests: CALLSTKCOUNT depth, CALLSTK$(n) frame names, CALLSTK file dump
GLOBAL failures AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL n AS LONG
    LOCAL waitk AS STRING
    failures = 0

    ' depth 1: only PBMAIN on the stack
    n = CALLSTKCOUNT
    IF n <> 1 THEN ConPrint "FAIL: PBMAIN depth=" & STR$(n): failures = failures + 1

    CALL TestA()

    IF failures = 0 THEN
        ConPrint "batch33: ALL PASS"
    ELSE
        ConPrint "batch33: FAILURES=" & STR$(failures)
    END IF

    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION

SUB TestA()
    LOCAL n AS LONG
    n = CALLSTKCOUNT
    IF n <> 2 THEN ConPrint "FAIL: TestA depth=" & STR$(n): failures = failures + 1
    IF CALLSTK$(1) <> "TESTA()" THEN ConPrint "FAIL: innermost=" & CALLSTK$(1): failures = failures + 1
    IF CALLSTK$(2) <> "PBMAIN()" THEN ConPrint "FAIL: frame2=" & CALLSTK$(2): failures = failures + 1
    IF CALLSTK$(99) <> "" THEN ConPrint "FAIL: OOR not empty": failures = failures + 1
    CALL TestB()
    ' depth back to 2 after TestB returns
    n = CALLSTKCOUNT
    IF n <> 2 THEN ConPrint "FAIL: after TestB depth=" & STR$(n): failures = failures + 1
END SUB

SUB TestB()
    LOCAL n AS LONG
    n = CALLSTKCOUNT
    IF n <> 3 THEN ConPrint "FAIL: TestB depth=" & STR$(n): failures = failures + 1
    CALLSTK "callstk.log"
END SUB


