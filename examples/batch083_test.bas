

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

' Batch 83: RETAIN$ function - retain matching substrings or ANY characters
FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL r AS STRING
    LOCAL waitk AS STRING
    LOCAL pass AS LONG
    pass = 0

    ConPrint "=== Batch 83: RETAIN$ function ==="

    ' Test 1: retain single substring
    s = "a1b2c3"
    r = RETAIN$(s, "1")
    ConPrint "Test 1: retain '1' from ['a1b2c3']"
    ConPrint "  Result: [" & r & "]"
    IF r = "1" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected '1')"
    END IF

    ' Test 2: retain multiple matching substrings
    s = "a1b1c1"
    r = RETAIN$(s, "1")
    ConPrint "Test 2: retain '1' from ['a1b1c1']"
    ConPrint "  Result: [" & r & "]"
    IF r = "111" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected '111')"
    END IF

    ' Test 3: ANY - retain only digits
    s = "hello123world456"
    r = RETAIN$(s, ANY "0123456789")
    ConPrint "Test 3: ANY retain digits from ['hello123world456']"
    ConPrint "  Result: [" & r & "]"
    IF r = "123456" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected '123456')"
    END IF

    ' Test 4: empty match string returns empty
    s = "abc"
    r = RETAIN$(s, "")
    ConPrint "Test 4: retain '' from ['abc']"
    ConPrint "  Result: [" & r & "]"
    IF r = "" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected empty)"
    END IF

    ConPrint "=== " & STR$(pass) & "/4 TESTS PASSED ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





