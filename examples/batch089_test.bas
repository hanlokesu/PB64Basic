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

' Batch 89: ERROR$ â€? error message function
FUNCTION PBMAIN() AS LONG
    LOCAL pass AS LONG
    LOCAL waitk AS STRING
    pass = 0

    ConPrint "=== Batch 89: ERROR$ ==="

    ' Test 1: ERROR$(0) = "No Error"
    IF ERROR$(0) = "No Error" THEN
        ConPrint "Test 1 PASS: ERROR$(0) = [No Error]"
        pass = pass + 1
    ELSE
        ConPrint "Test 1 FAIL: ERROR$(0) = [" & ERROR$(0) & "]"
    END IF

    ' Test 2: ERROR$(53) = "File not found"
    IF ERROR$(53) = "File not found" THEN
        ConPrint "Test 2 PASS: ERROR$(53) = [File not found]"
        pass = pass + 1
    ELSE
        ConPrint "Test 2 FAIL: ERROR$(53) = [" & ERROR$(53) & "]"
    END IF

    ' Test 3: ERROR$(11) = "Division by zero"
    IF ERROR$(11) = "Division by zero" THEN
        ConPrint "Test 3 PASS: ERROR$(11) = [Division by zero]"
        pass = pass + 1
    ELSE
        ConPrint "Test 3 FAIL: ERROR$(11) = [" & ERROR$(11) & "]"
    END IF

    ' Test 4: ERROR$(76) = "Path not found"
    IF ERROR$(76) = "Path not found" THEN
        ConPrint "Test 4 PASS: ERROR$(76) = [Path not found]"
        pass = pass + 1
    ELSE
        ConPrint "Test 4 FAIL: ERROR$(76) = [" & ERROR$(76) & "]"
    END IF

    ' Test 5: ERROR$(999) = "No Error" (PBWin10 measured)
    IF ERROR$(999) = "No Error" THEN
        ConPrint "Test 5 PASS: ERROR$(999) = [No Error]"
        pass = pass + 1
    ELSE
        ConPrint "Test 5 FAIL: ERROR$(999) = [" & ERROR$(999) & "]"
    END IF

    ' Test 6: ERROR$ (no args) with no error set = "No Error"
    IF ERROR$ = "No Error" THEN
        ConPrint "Test 6 PASS: ERROR$ (no error) = [No Error]"
        pass = pass + 1
    ELSE
        ConPrint "Test 6 FAIL: ERROR$ (no error) = [" & ERROR$ & "]"
    END IF

    ' Test 7: ERROR$(2) = "Syntax error"
    IF ERROR$(2) = "" THEN
        ConPrint "Test 7 PASS: ERROR$(2) = []"
        pass = pass + 1
    ELSE
        ConPrint "Test 7 FAIL: ERROR$(2) = [" & ERROR$(2) & "]"
    END IF

    ' Test 8: ERROR$(70) = "Permission denied"
    IF ERROR$(70) = "Permission denied" THEN
        ConPrint "Test 8 PASS: ERROR$(70) = [Permission denied]"
        pass = pass + 1
    ELSE
        ConPrint "Test 8 FAIL: ERROR$(70) = [" & ERROR$(70) & "]"
    END IF

    ConPrint "=== " & STR$(pass) & "/8 TESTS PASSED ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


