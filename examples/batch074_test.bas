

' --- PBWin10 stub branch: this sample exercises PowerBasilisk-only ---
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

' PowerBasilisk Enhanced - Batch 74 Test: XPRINT printer properties SET/GET
FUNCTION PBMAIN() AS LONG
    LOCAL v AS LONG
    LOCAL v2 AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    fails = 0

    ConPrint "=== Batch 74: XPRINT printer properties ==="

    XPRINT ATTACH DEFAULT

    XPRINT SET COPIES 3
    XPRINT GET COPIES TO v
    ConPrint "COPIES:" & STR$(v)
    IF v <> 3 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT SET ORIENTATION 2
    XPRINT GET ORIENTATION TO v
    ConPrint "ORIENTATION:" & STR$(v)
    IF v <> 2 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT SET QUALITY 3
    XPRINT GET QUALITY TO v
    ConPrint "QUALITY:" & STR$(v)
    IF v <> 3 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT SET DUPLEX 1
    XPRINT GET DUPLEX TO v
    ConPrint "DUPLEX:" & STR$(v)
    IF v <> 1 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT SET COLLATE 1
    XPRINT GET COLLATE TO v
    ConPrint "COLLATE:" & STR$(v)
    IF v <> 1 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT SET COLORMODE 1
    XPRINT GET COLORMODE TO v
    ConPrint "COLORMODE:" & STR$(v)
    IF v <> 1 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT SET PAGES 1, 5
    XPRINT GET PAGES TO v, v2
    ConPrint "PAGES:" & STR$(v) & "-" & STR$(v2)
    IF v <> 1 OR v2 <> 5 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT CLOSE

    ConPrint "=== Result: "
    IF fails = 0 THEN
        ConPrint "ALL PASS (7 SET/GET pairs)"
    ELSE
        ConPrint STR$(fails) & " FAILED"
    END IF

    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





