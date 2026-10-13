

' --- PBWin10 stub branch: this sample exercises PowerBasilisk-only ---
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

' Batch 84: REMAIN$ function - return portion after first match
FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL r AS STRING
    LOCAL waitk AS STRING
    LOCAL pass AS LONG
    pass = 0

    ConPrint "=== Batch 84: REMAIN$ function ==="

    ' Test 1: basic remain after space
    s = "hello world"
    r = REMAIN$(s, " ")
    ConPrint "Test 1: REMAIN$('hello world', ' ')"
    ConPrint "  Result: [" & r & "]"
    IF r = "world" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected 'world')"
    END IF

    ' Test 2: match not found returns empty
    s = "hello"
    r = REMAIN$(s, "xyz")
    ConPrint "Test 2: REMAIN$('hello', 'xyz') (not found)"
    ConPrint "  Result: [" & r & "]"
    IF r = "" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected empty)"
    END IF

    ' Test 3: with Start position
    s = "a1b2c3"
    r = REMAIN$(3, s, "b")
    ConPrint "Test 3: REMAIN$(3, 'a1b2c3', 'b')"
    ConPrint "  Result: [" & r & "]"
    IF r = "2c3" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected '2c3')"
    END IF

    ' Test 4: ANY - remain after first digit
    s = "hello123world"
    r = REMAIN$(s, ANY "0123456789")
    ConPrint "Test 4: REMAIN$('hello123world', ANY, digits)"
    ConPrint "  Result: [" & r & "]"
    IF r = "23world" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected '23world')"
    END IF

    ConPrint "=== " & STR$(pass) & "/4 TESTS PASSED ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





