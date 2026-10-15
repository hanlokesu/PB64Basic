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

' Batch 92: FRE() — free memory in bytes
FUNCTION PBMAIN() AS LONG
    LOCAL freemem AS QUAD
    LOCAL passed AS LONG
    passed = 1

    ' Test 1: FRE() returns a positive number (free physical memory)
    freemem = FRE
    IF freemem <= 0 THEN ConPrint "Test 1 FAIL: FRE() = " & STR$(freemem) & " expected > 0": passed = 0

    ' Test 2: FRE() returns a reasonable value (at least 1MB)
    IF freemem < 1048576 THEN ConPrint "Test 2 FAIL: FRE() = " & STR$(freemem) & " expected >= 1MB": passed = 0

    ' Test 3: FRE() with string argument (PB syntax: FRE("") triggers garbage collection)
    freemem = FRE("")
    IF freemem <= 0 THEN ConPrint "Test 3 FAIL: FRE("") = " & STR$(freemem) & " expected > 0": passed = 0

    ' Test 4: FRE() with numeric argument (PB syntax: FRE(0))
    freemem = FRE(0)
    IF freemem <= 0 THEN ConPrint "Test 4 FAIL: FRE(0) = " & STR$(freemem) & " expected > 0": passed = 0

    ' Test 5: Print the actual free memory value
    ConPrint "Free physical memory: " & STR$(FRE) & " bytes (" & STR$(FRE / 1048576) & " MB)"

    IF passed THEN
        ConPrint "=== 5/5 TESTS PASSED ==="
    ELSE
        ConPrint "=== SOME TESTS FAILED ==="
    END IF

    LOCAL waitk AS STRING
    waitk = WAITKEY$
END FUNCTION




#ENDIF
