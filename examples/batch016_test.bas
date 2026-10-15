

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

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL ok AS LONG
    LOCAL s AS STRING
    LOCAL w AS LONG
    LOCAL h AS LONG
    LOCAL x AS LONG
    LOCAL y AS LONG
    ok = 0

    ' --- MKx family: length + first byte (little-endian) ---
    s = MKI$(1000)
    IF LEN(s) <> 2 THEN ok = ok + 1
    IF ASC(s) <> 232 THEN ok = ok + 10          ' 0xE8 = low byte of 1000

    s = MKWRD$(1000)
    IF LEN(s) <> 2 THEN ok = ok + 100
    IF ASC(s) <> 232 THEN ok = ok + 1000

    s = MKL$(1000)
    IF LEN(s) <> 4 THEN ok = ok + 10000
    IF ASC(s) <> 232 THEN ok = ok + 100000

    s = MKDWD$(1000)
    IF LEN(s) <> 4 THEN ok = ok + 1000000
    IF ASC(s) <> 232 THEN ok = ok + 10000000

    s = MKQ$(1000)
    IF LEN(s) <> 8 THEN ok = ok + 100000000
    IF ASC(s) <> 232 THEN ok = ok + 1000000000

    s = MKCUR$(1000)
    IF LEN(s) <> 8 THEN ok = ok + 1

    s = MKCUX$(1000)
    IF LEN(s) <> 8 THEN ok = ok + 10

    s = MKS$(1000)
    IF LEN(s) <> 4 THEN ok = ok + 100

    s = MKD$(1000)
    IF LEN(s) <> 8 THEN ok = ok + 1000

    ' --- DESKTOP GET CLIENT / LOC / PPI ---
    DESKTOP GET CLIENT TO w, h
    IF w <= 0 OR h <= 0 THEN ok = ok + 10000

    DESKTOP GET LOC TO x, y
    IF x < 0 OR y < 0 THEN ok = ok + 100000

    ' DESKTOP GET PPI is a PB64Basic extension with no official PB
    ' equivalent; the PPI check is dropped for dual-compiler parity.

    IF ok = 0 THEN
        ConPrint "BATCH16 ALL PASS"
        FUNCTION = 0
    ELSE
        ConPrint "FAIL code=" & STR$(ok)
        FUNCTION = 1
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





