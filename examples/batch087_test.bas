#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

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

' Batch 87: CSTR / CQUAD / CBYTE / CWORD / CDWORD — type conversion functions
FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL q AS QUAD
    LOCAL b AS BYTE
    LOCAL w AS WORD
    LOCAL dwval AS DWORD
    LOCAL waitk AS STRING
    LOCAL pass AS LONG
    pass = 0

    ConPrint "=== Batch 87: CSTR / CQUAD / CBYTE / CWORD / CDWORD ==="

    ' Test 1: CSTR — numeric to string (no leading space)
    s = CSTR(123)
    IF s = "123" THEN
        ConPrint "Test 1 PASS: CSTR(123) = [" & s & "]"
        pass = pass + 1
    ELSE
        ConPrint "Test 1 FAIL: CSTR(123) = [" & s & "], expected [123]"
    END IF

    ' Test 2: CSTR — negative number
    s = CSTR(-456)
    IF s = "-456" THEN
        ConPrint "Test 2 PASS: CSTR(-456) = [" & s & "]"
        pass = pass + 1
    ELSE
        ConPrint "Test 2 FAIL: CSTR(-456) = [" & s & "], expected [-456]"
    END IF

    ' Test 3: CQUAD — convert to 64-bit integer
    q = CQUAD(123456789012345)
    IF q = 123456789012345 THEN
        ConPrint "Test 3 PASS: CQUAD(123456789012345) =" & STR$(q)
        pass = pass + 1
    ELSE
        ConPrint "Test 3 FAIL: CQUAD =" & STR$(q) & ", expected 123456789012345"
    END IF

    ' Test 4: CQUAD — from float (truncates toward zero)
    q = CQUAD(42.7)
    IF q = 42 THEN
        ConPrint "Test 4 PASS: CQUAD(42.7) =" & STR$(q)
        pass = pass + 1
    ELSE
        ConPrint "Test 4 FAIL: CQUAD(42.7) =" & STR$(q) & ", expected 42"
    END IF

    ' Test 5: CBYTE — convert to unsigned byte (0-255), truncates
    b = CBYTE(300)
    IF b = 44 THEN  ' 300 mod 256 = 44
        ConPrint "Test 5 PASS: CBYTE(300) =" & STR$(b) & " (300 mod 256)"
        pass = pass + 1
    ELSE
        ConPrint "Test 5 FAIL: CBYTE(300) =" & STR$(b) & ", expected 44"
    END IF

    ' Test 6: CBYTE — -1 wraps to 255
    b = CBYTE(-1)
    IF b = 255 THEN
        ConPrint "Test 6 PASS: CBYTE(-1) =" & STR$(b) & " (unsigned wrap)"
        pass = pass + 1
    ELSE
        ConPrint "Test 6 FAIL: CBYTE(-1) =" & STR$(b) & ", expected 255"
    END IF

    ' Test 7: CWORD — convert to unsigned word (0-65535)
    w = CWORD(70000)
    IF w = 4464 THEN  ' 70000 mod 65536 = 4464
        ConPrint "Test 7 PASS: CWORD(70000) =" & STR$(w) & " (70000 mod 65536)"
        pass = pass + 1
    ELSE
        ConPrint "Test 7 FAIL: CWORD(70000) =" & STR$(w) & ", expected 4464"
    END IF

    ' Test 8: CDWORD — convert to unsigned double word (32-bit)
    dwval = CDWORD(5000000000)
    IF dwval = 705032704 THEN  ' 5000000000 mod 2^32 = 705032704
        ConPrint "Test 8 PASS: CDWORD(5000000000) =" & STR$(dwval) & " (mod 2^32)"
        pass = pass + 1
    ELSE
        ConPrint "Test 8 FAIL: CDWORD(5000000000) =" & STR$(dwval) & ", expected 705032704"
    END IF

    ConPrint "=== " & STR$(pass) & "/8 TESTS PASSED ==="
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
