#COMPILE EXE

' batch327 self-check: EXTERN external data symbol (FORK EXTENSION).
' Links the bin2coff COFF object batch327_data.obj and reads the embedded
' 37-byte text block plus its 4-byte size symbol through EXTERN + PEEK$.
' Headless only: every assertion is counted, results go through ConPrint,
' no WAITKEY / MSGBOX / dialog.
' FORK-ONLY: PBWin 10 has no EXTERN keyword and rejects #LINK .obj (630).

#LINK "batch327_data.obj"
EXTERN b327data ALIAS "b327data" AS BYTE PTR
EXTERN b327data_size ALIAS "b327data_size" AS DWORD

DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

GLOBAL g_failures AS LONG

SUB CheckLong(BYVAL got AS LONG, BYVAL expected AS LONG, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=" + FORMAT$(got) + " expected=" + FORMAT$(expected) + $CRLF
    END IF
END SUB

SUB CheckStr(BYVAL got AS STRING, BYVAL expected AS STRING, BYVAL tag AS STRING)
    IF got <> expected THEN
        g_failures = g_failures + 1
        ConPrint "FAIL " + tag + " got=[" + got + "] expected=[" + expected + "]" + $CRLF
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL txt AS STRING
    LOCAL addr AS QUAD
    LOCAL n AS DWORD
    n = b327data_size
    CheckLong CLNG(n), 37, "size"
    txt = PEEK$(b327data, b327data_size)
    CheckStr txt, "PB64Basic EXTERN embedded data block.", "data"
    addr = b327data
    IF addr = 0 THEN
        g_failures = g_failures + 1
        ConPrint "FAIL addr is zero" + $CRLF
    END IF
    ConPrint "batch327 failures=" + FORMAT$(g_failures) + $CRLF
    FUNCTION = g_failures
END FUNCTION
