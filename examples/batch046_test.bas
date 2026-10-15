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

' PB64Basic Enhanced - batch 46: MEMORY COPY / SWAP / FILL
GLOBAL failures AS LONG

SUB Check(cond AS LONG, msg AS STRING)
    IF cond = 0 THEN
        ConPrint "FAIL: " & msg
        failures = failures + 1
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    DIM buf(20) AS BYTE
    DIM src(4) AS BYTE
    LOCAL i AS LONG
    LOCAL n AS LONG
    failures = 0

    ' init src = {10, 20, 30, 40, 50}
    FOR i = 0 TO 4
        src(i) = (i + 1) * 10
    NEXT i

    ' MEMORY COPY: src -> buf
    MEMORY COPY VARPTR(src(0)), VARPTR(buf(0)), 5
    n = 1
    FOR i = 0 TO 4
        IF buf(i) <> (i + 1) * 10 THEN n = 0
    NEXT i
    Check(n = 1, "mem copy")

    ' MEMORY FILL BYTE: buf(0..4) = 0xAB
    MEMORY FILL VARPTR(buf(0)), 5, BYTE &HAB
    Check(buf(0) = 171 AND buf(1) = 171 AND buf(2) = 171 AND buf(3) = 171 AND buf(4) = 171, "mem fill byte")

    ' MEMORY SWAP: swap buf and src
    MEMORY SWAP VARPTR(buf(0)), VARPTR(src(0)), 5
    Check(src(0) = 171 AND src(1) = 171 AND src(2) = 171 AND src(3) = 171 AND src(4) = 171, "mem swap dst")
    Check(buf(0) = 10 AND buf(1) = 20 AND buf(2) = 30 AND buf(3) = 40 AND buf(4) = 50, "mem swap src")

    ' MEMORY FILL WORD: 3 words of 0x0102 -> bytes 02 01 02 01 02 01
    MEMORY FILL VARPTR(buf(0)), 3, WORD &H0102
    Check(buf(0) = 2 AND buf(1) = 1 AND buf(2) = 2 AND buf(3) = 1 AND buf(4) = 2, "mem fill word")

    ' MEMORY FILL DWORD: 2 dwords of &H04030201
    MEMORY FILL VARPTR(buf(0)), 2, DWORD &H04030201
    Check(buf(0) = 1 AND buf(1) = 2 AND buf(2) = 3 AND buf(3) = 4, "mem fill dword")

    ' MEMORY FILL with string pattern: "AB" over 5 bytes -> A B A B A
    MEMORY FILL VARPTR(buf(0)), 5, "AB"
    Check(buf(0) = ASC("A") AND buf(1) = ASC("B") AND buf(2) = ASC("A") AND buf(3) = ASC("B") AND buf(4) = ASC("A"), "mem fill str")

    IF failures = 0 THEN
        ConPrint "batch46: ALL PASS"
    ELSE
        ConPrint "batch46: FAILURES=" & STR$(failures)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

