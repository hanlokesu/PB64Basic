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

' Batch 95: SEC/CSC/COT/SECH/CSCH — reciprocal trig + hyperbolic functions
FUNCTION PBMAIN() AS LONG
    LOCAL passed AS LONG
    LOCAL v AS DOUBLE
    passed = 1

    ' Test 1: SEC(0) = 1/cos(0) = 1
    v = SEC(0)
    IF ABS(v - 1) > 0.0001 THEN ConPrint "Test 1 FAIL: SEC(0) = " & STR$(v) & " expected 1": passed = 0

    ' Test 2: CSC(pi/2) = 1/sin(pi/2) = 1 (use 1.5708)
    v = CSC(1.5708)
    IF ABS(v - 1) > 0.001 THEN ConPrint "Test 2 FAIL: CSC(pi/2) = " & STR$(v) & " expected ~1": passed = 0

    ' Test 3: COT(pi/4) = 1/tan(pi/4) = 1 (use 0.7854)
    v = COT(0.7854)
    IF ABS(v - 1) > 0.001 THEN ConPrint "Test 3 FAIL: COT(pi/4) = " & STR$(v) & " expected ~1": passed = 0

    ' Test 4: SECH(0) = 1/cosh(0) = 1
    v = SECH(0)
    IF ABS(v - 1) > 0.0001 THEN ConPrint "Test 4 FAIL: SECH(0) = " & STR$(v) & " expected 1": passed = 0

    ' Test 5: SECH(1) = 1/cosh(1) ~ 0.6481
    v = SECH(1)
    IF v < 0.64 OR v > 0.66 THEN ConPrint "Test 5 FAIL: SECH(1) = " & STR$(v) & " expected ~0.6481": passed = 0

    ' Test 6: CSCH(1) = 1/sinh(1) ~ 0.8509
    v = CSCH(1)
    IF v < 0.84 OR v > 0.86 THEN ConPrint "Test 6 FAIL: CSCH(1) = " & STR$(v) & " expected ~0.8509": passed = 0

    ' Test 7: SEC(1) = 1/cos(1) ~ 1.8508
    v = SEC(1)
    IF v < 1.84 OR v > 1.86 THEN ConPrint "Test 7 FAIL: SEC(1) = " & STR$(v) & " expected ~1.8508": passed = 0

    ' Test 8: CSC(1) = 1/sin(1) ~ 1.1884
    v = CSC(1)
    IF v < 1.18 OR v > 1.20 THEN ConPrint "Test 8 FAIL: CSC(1) = " & STR$(v) & " expected ~1.1884": passed = 0

    ' Test 9: COT(1) = 1/tan(1) ~ 0.6421
    v = COT(1)
    IF v < 0.63 OR v > 0.65 THEN ConPrint "Test 9 FAIL: COT(1) = " & STR$(v) & " expected ~0.6421": passed = 0

    ' Test 10: verify SEC*COS = 1
    v = SEC(0.5) * COS(0.5)
    IF ABS(v - 1) > 0.0001 THEN ConPrint "Test 10 FAIL: SEC(0.5)*COS(0.5) = " & STR$(v) & " expected 1": passed = 0

    IF passed THEN
        ConPrint "=== 10/10 TESTS PASSED ==="
    ELSE
        ConPrint "=== SOME TESTS FAILED ==="
    END IF

    LOCAL waitk AS STRING
    waitk = WAITKEY$
END FUNCTION

FUNCTION SEC(BYVAL x AS DOUBLE) AS DOUBLE
    IF COS(x) <> 0 THEN
        FUNCTION = 1 / COS(x)
    ELSE
        FUNCTION = 0
    END IF
END FUNCTION



#ENDIF
