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

' Batch 99: CHRBYTES + FUNCNAME$ test
FUNCTION PBMAIN() AS LONG
    LOCAL s AS STRING
    LOCAL waitk AS STRING
    LOCAL result AS LONG

    ConPrint "=== Batch 99: CHRBYTES + FUNCNAME$ ==="

    ' Test 1: CHRBYTES with dynamic string (should return 1)
    s = "hello"
    result = CHRBYTES(s)
    ConPrint "Test 1: CHRBYTES(dynamic string) = " & STR$(result) & " (expected 1)"
    IF result = 1 THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF

    ' Test 2: CHRBYTES with empty string
    s = ""
    result = CHRBYTES(s)
    ConPrint "Test 2: CHRBYTES(empty string) = " & STR$(result) & " (expected 1)"
    IF result = 1 THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF

    ' Test 3: FUNCNAME$ inside PBMAIN (should return "PBMAIN")
    s = FUNCNAME$
    ConPrint "Test 3: FUNCNAME$ in PBMAIN = [" & s & "] (expected PBMAIN)"
    IF s = "PBMAIN" THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF

    ' Test 4: FUNCNAME$ bare form (no parens)
    s = FUNCNAME
    ConPrint "Test 4: FUNCNAME (bare) = [" & s & "] (expected PBMAIN)"
    IF s = "PBMAIN" THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF

    ' Test 5: Call a sub and check FUNCNAME$ inside it
    CALL TestSub()

    ConPrint ""
    ConPrint "=== Batch 99 tests complete ==="
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

SUB TestSub()
    LOCAL s AS STRING
    s = FUNCNAME$
    ConPrint "Test 5: FUNCNAME$ in TestSub = [" & s & "] (expected TESTSUB)"
    IF s = "TESTSUB" THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF
END SUB


#ENDIF
