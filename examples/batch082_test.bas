

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

' Batch 82: REMOVE$ function - remove substrings or ANY characters
FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL r AS STRING
    LOCAL waitk AS STRING
    LOCAL pass AS LONG
    pass = 0

    ConPrint "=== Batch 82: REMOVE$ function ==="

    ' Test 1: remove substring
    s = "the cat sat on the mat"
    r = REMOVE$(s, "the ")
    ConPrint "Test 1: remove 'the ' from ['the cat sat on the mat']"
    ConPrint "  Result: [" & r & "]"
    IF r = "cat sat on mat" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected 'cat sat on mat')"
    END IF

    ' Test 2: remove substring not found
    s = "hello world"
    r = REMOVE$(s, "xyz")
    ConPrint "Test 2: remove 'xyz' (not found) from ['hello world']"
    ConPrint "  Result: [" & r & "]"
    IF r = "hello world" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected 'hello world')"
    END IF

    ' Test 3: ANY - remove any character in match
    s = "hello world"
    r = REMOVE$(s, ANY "lo")
    ConPrint "Test 3: ANY remove 'l','o' from ['hello world']"
    ConPrint "  Result: [" & r & "]"
    IF r = "he wrd" THEN
        ConPrint "  PASS"
        pass = pass + 1
    ELSE
        ConPrint "  FAIL (expected 'he wrd')"
    END IF

    ' Test 4: remove all occurrences
    s = "aaaa"
    r = REMOVE$(s, "aa")
    ConPrint "Test 4: remove 'aa' from ['aaaa']"
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





