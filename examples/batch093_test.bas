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

' Batch 93: ASIN/ACOS/SINH/COSH/TANH — inverse trig + hyperbolic functions
FUNCTION PBMAIN() AS LONG
    LOCAL passed AS LONG
    LOCAL v AS DOUBLE
    passed = 1

    ' Test 1: ASIN(0) = 0
    v = ASIN(0)
    IF ABS(v) > 0.0001 THEN ConPrint "Test 1 FAIL: ASIN(0) = " & STR$(v) & " expected 0": passed = 0

    ' Test 2: ASIN(1) = pi/2 (~1.5708)
    v = ASIN(1)
    IF v < 1.57 OR v > 1.572 THEN ConPrint "Test 2 FAIL: ASIN(1) = " & STR$(v) & " expected ~1.5708": passed = 0

    ' Test 3: ACOS(1) = 0
    v = ACOS(1)
    IF ABS(v) > 0.0001 THEN ConPrint "Test 3 FAIL: ACOS(1) = " & STR$(v) & " expected 0": passed = 0

    ' Test 4: ACOS(0) = pi/2 (~1.5708)
    v = ACOS(0)
    IF v < 1.57 OR v > 1.572 THEN ConPrint "Test 4 FAIL: ACOS(0) = " & STR$(v) & " expected ~1.5708": passed = 0

    ' Test 5: SINH(0) = 0
    v = SINH(0)
    IF ABS(v) > 0.0001 THEN ConPrint "Test 5 FAIL: SINH(0) = " & STR$(v) & " expected 0": passed = 0

    ' Test 6: SINH(1) ~ 1.1752
    v = SINH(1)
    IF v < 1.17 OR v > 1.18 THEN ConPrint "Test 6 FAIL: SINH(1) = " & STR$(v) & " expected ~1.1752": passed = 0

    ' Test 7: COSH(0) = 1
    v = COSH(0)
    IF ABS(v - 1) > 0.0001 THEN ConPrint "Test 7 FAIL: COSH(0) = " & STR$(v) & " expected 1": passed = 0

    ' Test 8: COSH(1) ~ 1.5431
    v = COSH(1)
    IF v < 1.54 OR v > 1.55 THEN ConPrint "Test 8 FAIL: COSH(1) = " & STR$(v) & " expected ~1.5431": passed = 0

    ' Test 9: TANH(0) = 0
    v = TANH(0)
    IF ABS(v) > 0.0001 THEN ConPrint "Test 9 FAIL: TANH(0) = " & STR$(v) & " expected 0": passed = 0

    ' Test 10: TANH(1) ~ 0.7616
    v = TANH(1)
    IF v < 0.76 OR v > 0.77 THEN ConPrint "Test 10 FAIL: TANH(1) = " & STR$(v) & " expected ~0.7616": passed = 0

    IF passed THEN
        ConPrint "=== 10/10 TESTS PASSED ==="
    ELSE
        ConPrint "=== SOME TESTS FAILED ==="
    END IF

    LOCAL waitk AS STRING
    waitk = WAITKEY$
END FUNCTION

FUNCTION ASIN(BYVAL x AS DOUBLE) AS DOUBLE
    IF x >= -1 AND x <= 1 THEN
        FUNCTION = ATN(x / SQR(1 - x * x))
    ELSE
        FUNCTION = 0
    END IF
END FUNCTION



#ENDIF
