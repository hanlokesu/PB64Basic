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

' Batch 91: Type conversion aliases — CFLT, CLNGINT, CUINT, CULNG
FUNCTION PBMAIN() AS LONG
    LOCAL n AS LONG
    LOCAL s AS SINGLE
    LOCAL passed AS LONG
    passed = 1

    ' Test 1: CFLT(3.14) = 3.14 (Single)
    s = CFLT(3.14)
    IF s < 3.13 OR s > 3.15 THEN ConPrint "Test 1 FAIL: CFLT(3.14) = " & STR$(s) & " expected ~3.14": passed = 0

    ' Test 2: CFLT(42) = 42.0 (integer to Single)
    s = CFLT(42)
    IF s <> 42.0 THEN ConPrint "Test 2 FAIL: CFLT(42) = " & STR$(s) & " expected 42.0": passed = 0

    ' Test 3: CLNGINT(3.7) = 4 (banker's rounding, same as CLNG)
    n = CLNGINT(3.7)
    IF n <> 4 THEN ConPrint "Test 3 FAIL: CLNGINT(3.7) = " & STR$(n) & " expected 4": passed = 0

    ' Test 4: CLNGINT(-2.3) = -2
    n = CLNGINT(-2.3)
    IF n <> -2 THEN ConPrint "Test 4 FAIL: CLNGINT(-2.3) = " & STR$(n) & " expected -2": passed = 0

    ' Test 5: CUINT(100) = 100 (unsigned int alias)
    n = CUINT(100)
    IF n <> 100 THEN ConPrint "Test 5 FAIL: CUINT(100) = " & STR$(n) & " expected 100": passed = 0

    ' Test 6: CUINT(5.5) = 6 (rounding)
    n = CUINT(5.5)
    IF n <> 6 THEN ConPrint "Test 6 FAIL: CUINT(5.5) = " & STR$(n) & " expected 6": passed = 0

    ' Test 7: CULNG(99999) = 99999 (unsigned long alias)
    n = CULNG(99999)
    IF n <> 99999 THEN ConPrint "Test 7 FAIL: CULNG(99999) = " & STR$(n) & " expected 99999": passed = 0

    ' Test 8: CULNG(0.4) = 0 (round down)
    n = CULNG(0.4)
    IF n <> 0 THEN ConPrint "Test 8 FAIL: CULNG(0.4) = " & STR$(n) & " expected 0": passed = 0

    IF passed THEN
        ConPrint "=== 8/8 TESTS PASSED ==="
    ELSE
        ConPrint "=== SOME TESTS FAILED ==="
    END IF

    LOCAL waitk AS STRING
    waitk = WAITKEY$
END FUNCTION

FUNCTION CFLT(BYVAL v AS LONG) AS DOUBLE
    FUNCTION = v
END FUNCTION



#ENDIF
