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

' Batch 88: ISTRUE / ISFALSE / ISEVEN / ISODD — boolean predicate functions
FUNCTION PBMAIN() AS LONG
    LOCAL pass AS LONG
    LOCAL waitk AS STRING
    pass = 0

    ConPrint "=== Batch 88: ISTRUE / ISFALSE / ISEVEN / ISODD ==="

    ' Test 1: ISTRUE — non-zero returns -1 (PB TRUE)
    IF ISTRUE(42) = -1 THEN
        ConPrint "Test 1 PASS: ISTRUE(42) = -1"
        pass = pass + 1
    ELSE
        ConPrint "Test 1 FAIL: ISTRUE(42) =" & STR$(ISTRUE(42)) & ", expected -1"
    END IF

    ' Test 2: ISTRUE — zero returns 0 (PB FALSE)
    IF ISTRUE(0) = 0 THEN
        ConPrint "Test 2 PASS: ISTRUE(0) = 0"
        pass = pass + 1
    ELSE
        ConPrint "Test 2 FAIL: ISTRUE(0) =" & STR$(ISTRUE(0)) & ", expected 0"
    END IF

    ' Test 3: ISFALSE — zero returns -1
    IF ISFALSE(0) = -1 THEN
        ConPrint "Test 3 PASS: ISFALSE(0) = -1"
        pass = pass + 1
    ELSE
        ConPrint "Test 3 FAIL: ISFALSE(0) =" & STR$(ISFALSE(0)) & ", expected -1"
    END IF

    ' Test 4: ISFALSE — non-zero returns 0
    IF ISFALSE(99) = 0 THEN
        ConPrint "Test 4 PASS: ISFALSE(99) = 0"
        pass = pass + 1
    ELSE
        ConPrint "Test 4 FAIL: ISFALSE(99) =" & STR$(ISFALSE(99)) & ", expected 0"
    END IF

    ' Test 5: ISEVEN — even number returns -1
    IF ISEVEN(10) = -1 THEN
        ConPrint "Test 5 PASS: ISEVEN(10) = -1"
        pass = pass + 1
    ELSE
        ConPrint "Test 5 FAIL: ISEVEN(10) =" & STR$(ISEVEN(10)) & ", expected -1"
    END IF

    ' Test 6: ISEVEN — odd number returns 0
    IF ISEVEN(7) = 0 THEN
        ConPrint "Test 6 PASS: ISEVEN(7) = 0"
        pass = pass + 1
    ELSE
        ConPrint "Test 6 FAIL: ISEVEN(7) =" & STR$(ISEVEN(7)) & ", expected 0"
    END IF

    ' Test 7: ISODD — odd number returns -1
    IF ISODD(15) = -1 THEN
        ConPrint "Test 7 PASS: ISODD(15) = -1"
        pass = pass + 1
    ELSE
        ConPrint "Test 7 FAIL: ISODD(15) =" & STR$(ISODD(15)) & ", expected -1"
    END IF

    ' Test 8: ISODD — even number returns 0
    IF ISODD(8) = 0 THEN
        ConPrint "Test 8 PASS: ISODD(8) = 0"
        pass = pass + 1
    ELSE
        ConPrint "Test 8 FAIL: ISODD(8) =" & STR$(ISODD(8)) & ", expected 0"
    END IF

    ' Test 9: ISTRUE with negative value
    IF ISTRUE(-5) = -1 THEN
        ConPrint "Test 9 PASS: ISTRUE(-5) = -1"
        pass = pass + 1
    ELSE
        ConPrint "Test 9 FAIL: ISTRUE(-5) =" & STR$(ISTRUE(-5)) & ", expected -1"
    END IF

    ' Test 10: ISEVEN with zero (zero is even)
    IF ISEVEN(0) = -1 THEN
        ConPrint "Test 10 PASS: ISEVEN(0) = -1 (zero is even)"
        pass = pass + 1
    ELSE
        ConPrint "Test 10 FAIL: ISEVEN(0) =" & STR$(ISEVEN(0)) & ", expected -1"
    END IF

    ConPrint "=== " & STR$(pass) & "/10 TESTS PASSED ==="
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

FUNCTION ISEVEN(BYVAL n AS LONG) AS LONG
    IF (n MOD 2) = 0 THEN
        FUNCTION = -1
    ELSE
        FUNCTION = 0
    END IF
END FUNCTION



#ENDIF
