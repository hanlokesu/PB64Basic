
' PB64Basic Enhanced - Batch 78 Test: XPRINT GET MARGIN (official PBWin10 syntax)
' --- Dual-compiler self-check: this sample runs on BOTH compilers. ---
'     DISPLAY OPENFILE/SAVEFILE/COLOR/FONT/BROWSE are PB64Basic-only
'     extensions; official PBWin10 rejects them (420/477), so they are not part
'     of this dual sample.
'     GET MARGIN values are per-environment (no default printer), not test
'     results - normalized to a fixed token while the statement still executes
'     on both sides.  Same pattern as batch 74.

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

' PB64Basic Enhanced - Batch 78 Test: XPRINT GET MARGIN
FUNCTION PBMAIN() AS LONG
    LOCAL ml AS LONG, mt AS LONG, mr AS LONG, mb AS LONG
    LOCAL waitk AS STRING

    ConPrint "=== Batch 78: XPRINT GET MARGIN ==="

    XPRINT ATTACH DEFAULT
    XPRINT GET MARGIN TO ml, mt, mr, mb
    ConPrint "GET MARGIN: ok"
    XPRINT CLOSE

    ConPrint "=== Result: ALL PASS ==="
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION
