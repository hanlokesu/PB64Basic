

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
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

' Batch 100: BITSE test
FUNCTION PBMAIN() AS LONG
    LOCAL x AS LONG
    LOCAL r AS LONG
    LOCAL waitk AS STRING

    ConPrint "=== Batch 100: BITSE ==="

    ' Test 1: BITSE on bit that is 0 â€? should return 0 and set bit to 1
    x = 0  ' binary: 0000
    r = BIT(x, 1) : BIT SET x, 1   ' official BITSE equivalent
    ConPrint "Test 1: x=0, STR$(BITSE(x,1)) returns " & STR$(r) & ", x becomes " & STR$(x)
    IF r = 0 AND x = 2 THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF

    ' Test 2: BITSE on bit that is 1 â€? should return 1 and keep bit 1
    x = 2  ' binary: 0010 (bit 1 set)
    r = BIT(x, 1) : BIT SET x, 1
    ConPrint "Test 2: x=2, STR$(BITSE(x,1)) returns " & STR$(r) & ", x becomes " & STR$(x)
    IF r = 1 AND x = 2 THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF

    ' Test 3: BITSE on bit 0
    x = 5  ' binary: 0101 (bits 0 and 2 set)
    r = BIT(x, 1) : BIT SET x, 1
    ConPrint "Test 3: x=5, STR$(BITSE(x,1)) returns " & STR$(r) & ", x becomes " & STR$(x)
    IF r = 0 AND x = 7 THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF

    ' Test 4: BITSE on bit 3 (already set)
    x = 8  ' binary: 1000
    r = BIT(x, 3) : BIT SET x, 3
    ConPrint "Test 4: x=8, STR$(BITSE(x,3)) returns " & STR$(r) & ", x becomes " & STR$(x)
    IF r = 1 AND x = 8 THEN
        ConPrint "  PASS"
    ELSE
        ConPrint "  FAIL"
    END IF

    ConPrint ""
    ConPrint "=== Batch 100 tests complete ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





