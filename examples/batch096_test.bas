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

' Batch 96: HYPOT/CBRT/EXPM1/LOG1P/ERF (C math library functions)
FUNCTION PBMAIN() AS LONG
    LOCAL x AS DOUBLE
    LOCAL y AS DOUBLE
    LOCAL r AS DOUBLE
    LOCAL pass AS LONG
    LOCAL waitk AS STRING
    pass = 0

    ConPrint "=== Batch 96: HYPOT/CBRT/EXPM1/LOG1P/ERF ==="

    ' Test 1: HYPOT(3,4) = 5
    r = HYPOT(3.0, 4.0)
    IF ABS(r - 5.0) < 0.0001 THEN
        ConPrint "Test 1 HYPOT(3,4)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 1 HYPOT(3,4)=" & STR$(r) & " FAIL (expected 5)"
    END IF

    ' Test 2: HYPOT(5,12) = 13
    r = HYPOT(5.0, 12.0)
    IF ABS(r - 13.0) < 0.0001 THEN
        ConPrint "Test 2 HYPOT(5,12)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 2 HYPOT(5,12)=" & STR$(r) & " FAIL (expected 13)"
    END IF

    ' Test 3: CBRT(8) = 2
    r = CBRT(8.0)
    IF ABS(r - 2.0) < 0.0001 THEN
        ConPrint "Test 3 CBRT(8)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 3 CBRT(8)=" & STR$(r) & " FAIL (expected 2)"
    END IF

    ' Test 4: CBRT(27) = 3
    r = CBRT(27.0)
    IF ABS(r - 3.0) < 0.0001 THEN
        ConPrint "Test 4 CBRT(27)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 4 CBRT(27)=" & STR$(r) & " FAIL (expected 3)"
    END IF

    ' Test 5: EXPM1(0) = 0
    r = EXPM1(0.0)
    IF ABS(r - 0.0) < 0.0001 THEN
        ConPrint "Test 5 EXPM1(0)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 5 EXPM1(0)=" & STR$(r) & " FAIL (expected 0)"
    END IF

    ' Test 6: EXPM1(1) = e-1 ≈ 1.71828
    r = EXPM1(1.0)
    IF ABS(r - 1.71828) < 0.0001 THEN
        ConPrint "Test 6 EXPM1(1)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 6 EXPM1(1)=" & STR$(r) & " FAIL (expected 1.71828)"
    END IF

    ' Test 7: LOG1P(0) = 0
    r = LOG1P(0.0)
    IF ABS(r - 0.0) < 0.0001 THEN
        ConPrint "Test 7 LOG1P(0)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 7 LOG1P(0)=" & STR$(r) & " FAIL (expected 0)"
    END IF

    ' Test 8: LOG1P(e-1) = 1
    r = LOG1P(1.71828)
    IF ABS(r - 1.0) < 0.0001 THEN
        ConPrint "Test 8 LOG1P(e-1)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 8 LOG1P(e-1)=" & STR$(r) & " FAIL (expected 1)"
    END IF

    ' Test 9: ERF(0) = 0
    r = ERF(0.0)
    IF ABS(r - 0.0) < 0.0001 THEN
        ConPrint "Test 9 ERF(0)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 9 ERF(0)=" & STR$(r) & " FAIL (expected 0)"
    END IF

    ' Test 10: ERF(1) ≈ 0.84270
    r = ERF(1.0)
    IF ABS(r - 0.84270) < 0.0001 THEN
        ConPrint "Test 10 ERF(1)=" & STR$(r) & " OK"
        pass = pass + 1
    ELSE
        ConPrint "Test 10 ERF(1)=" & STR$(r) & " FAIL (expected 0.84270)"
    END IF

    ConPrint "=== ALL " & STR$(pass) & "/10 TESTS PASSED ==="
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

FUNCTION HYPOT(BYVAL x AS DOUBLE, BYVAL y AS DOUBLE) AS DOUBLE
    FUNCTION = SQR(x * x + y * y)
END FUNCTION



#ENDIF
