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

' Batch 94: ATN2/ASINH/ACOSH/ATANH/COTH — more inverse trig + hyperbolic
FUNCTION PBMAIN() AS LONG
    LOCAL passed AS LONG
    LOCAL v AS DOUBLE
    passed = 1

    ' Test 1: ATN2(0, 1) = 0
    v = ATN2(0, 1)
    IF ABS(v) > 0.0001 THEN ConPrint "Test 1 FAIL: ATN2(0,1) = " & STR$(v) & " expected 0": passed = 0

    ' Test 2: ATN2(1, 0) = pi/2 (~1.5708)
    v = ATN2(1, 0)
    IF v < 1.57 OR v > 1.572 THEN ConPrint "Test 2 FAIL: ATN2(1,0) = " & STR$(v) & " expected ~1.5708": passed = 0

    ' Test 3: ATN2(1, 1) = pi/4 (~0.7854)
    v = ATN2(1, 1)
    IF v < 0.785 OR v > 0.786 THEN ConPrint "Test 3 FAIL: ATN2(1,1) = " & STR$(v) & " expected ~0.7854": passed = 0

    ' Test 4: ASINH(0) = 0
    v = ASINH(0)
    IF ABS(v) > 0.0001 THEN ConPrint "Test 4 FAIL: ASINH(0) = " & STR$(v) & " expected 0": passed = 0

    ' Test 5: ASINH(1) ~ 0.8814
    v = ASINH(1)
    IF v < 0.88 OR v > 0.89 THEN ConPrint "Test 5 FAIL: ASINH(1) = " & STR$(v) & " expected ~0.8814": passed = 0

    ' Test 6: ACOSH(1) = 0
    v = ACOSH(1)
    IF ABS(v) > 0.0001 THEN ConPrint "Test 6 FAIL: ACOSH(1) = " & STR$(v) & " expected 0": passed = 0

    ' Test 7: ACOSH(2) ~ 1.3170
    v = ACOSH(2)
    IF v < 1.31 OR v > 1.32 THEN ConPrint "Test 7 FAIL: ACOSH(2) = " & STR$(v) & " expected ~1.3170": passed = 0

    ' Test 8: ATANH(0) = 0
    v = ATANH(0)
    IF ABS(v) > 0.0001 THEN ConPrint "Test 8 FAIL: ATANH(0) = " & STR$(v) & " expected 0": passed = 0

    ' Test 9: ATANH(0.5) ~ 0.5493
    v = ATANH(0.5)
    IF v < 0.54 OR v > 0.56 THEN ConPrint "Test 9 FAIL: ATANH(0.5) = " & STR$(v) & " expected ~0.5493": passed = 0

    ' Test 10: COTH(1) = 1/tanh(1) ~ 1.3130
    v = COTH(1)
    IF v < 1.31 OR v > 1.32 THEN ConPrint "Test 10 FAIL: COTH(1) = " & STR$(v) & " expected ~1.3130": passed = 0

    IF passed THEN
        ConPrint "=== 10/10 TESTS PASSED ==="
    ELSE
        ConPrint "=== SOME TESTS FAILED ==="
    END IF

    LOCAL waitk AS STRING
    waitk = WAITKEY$
END FUNCTION

FUNCTION ATN2(BYVAL y AS DOUBLE, BYVAL x AS DOUBLE) AS DOUBLE
    LOCAL r AS DOUBLE
    IF x = 0 THEN
        IF y > 0 THEN
            r = 1.5707963
        ELSE
            r = -1.5707963
        END IF
    ELSE
        r = ATN(y / x)
        IF x < 0 THEN
            IF y >= 0 THEN
                r = r + 3.1415926
            ELSE
                r = r - 3.1415926
            END IF
        END IF
    END IF
    FUNCTION = r
END FUNCTION



#ENDIF
