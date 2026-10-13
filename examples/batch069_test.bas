

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

' PowerBasilisk Enhanced - Batch 69 Test: XPRINT drawing + text + attributes
FUNCTION PBMAIN() AS LONG
    LOCAL hdc AS LONG
    LOCAL attached AS STRING
    LOCAL px AS LONG, py AS LONG
    LOCAL col AS LONG
    LOCAL pix AS LONG
    LOCAL ta AS LONG
    LOCAL waitk AS STRING
    LOCAL fails AS LONG
    fails = 0

    ConPrint "=== Batch 69: XPRINT drawing + text ==="

    XPRINT ATTACH DEFAULT
    XPRINT GET ATTACH TO attached
    ConPrint "GET ATTACH:" & attached
    IF attached = "" THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT GET DC TO hdc
    IF hdc = 0 THEN fails = fails + 1 : ConPrint "  FAIL: dc zero"

    XPRINT SET POS (100, 100)
    XPRINT GET POS TO px, py
    ConPrint "SET/GET POS:" & STR$(px) & STR$(py)
    IF px <> 100 OR py <> 100 THEN fails = fails + 1 : ConPrint "  FAIL"

    XPRINT COLOR 255
    ' XPRINT GET COLOR has no official page; color is set-only.
    ConPrint "SET COLOR: done"

    XPRINT WIDTH 3
    XPRINT STYLE 0
    XPRINT LINE (10, 10)-(200, 200)
    XPRINT BOX (10, 10)-(200, 200)
    ConPrint "LINE/BOX: done"

    XPRINT SET PIXEL (50, 50), 255
    XPRINT GET PIXEL (50, 50) TO pix
    ConPrint "SET/GET PIXEL:" & STR$(pix)
    IF pix = -1 THEN fails = fails + 1 : ConPrint "  FAIL (CLR_INVALID)"

    ' XPRINT SET/GET TEXTALIGN have no official pages; skipped.
    ConPrint "TEXTALIGN: skipped (no official page)"

    XPRINT SET POS (50, 300)
    XPRINT PRINT "Hello from XPRINT!"
    ConPrint "XPRINT PRINT: done"

    XPRINT FORMFEED
    XPRINT GET POS TO px, py
    ConPrint "FORMFEED resets pos:" & STR$(px) & STR$(py)

    XPRINT CANCEL
    ConPrint "CANCEL: done"

    XPRINT CLOSE
    XPRINT GET ATTACH TO attached
    IF attached <> "" THEN fails = fails + 1 : ConPrint "  FAIL: still attached after close"

    ConPrint "=== Result: "
    IF fails = 0 THEN
        ConPrint "ALL PASS"
    ELSE
        ConPrint STR$(fails) & " FAILED"
    END IF

    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





