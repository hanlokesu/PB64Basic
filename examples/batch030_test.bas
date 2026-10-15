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

' PB64Basic Enhanced - Batch 30: ASMDATA / END ASMDATA read-only data blocks
' Verifies: DB/DW/DD/DQ byte layout, ANSI + WIDE string literals, CODEPTR() address,
' packed unaligned offsets, read-back via PEEK.
ASMDATA ABC
    DB 5, 2, 3
    DB 7, 8, 9
    DW 2, 3
    DD &H12345678
    DQ 1234567890
    DB "Hi", 0
    DW "AB"
END ASMDATA

FUNCTION PBMAIN() AS LONG
    LOCAL p AS QUAD
    LOCAL pass AS LONG
    LOCAL waitk AS STRING
    pass = 1

    p = CODEPTR(ABC)

    ' DB 5,2,3 at offsets 0..2
    IF PEEK(BYTE, p) <> 5 THEN pass = 0
    IF PEEK(BYTE, p + 1) <> 2 THEN pass = 0
    IF PEEK(BYTE, p + 2) <> 3 THEN pass = 0
    ' DB 7,8,9 at offsets 3..5
    IF PEEK(BYTE, p + 3) <> 7 THEN pass = 0
    IF PEEK(BYTE, p + 4) <> 8 THEN pass = 0
    IF PEEK(BYTE, p + 5) <> 9 THEN pass = 0
    ' DW 2,3 at offsets 6..9 (LE)
    IF PEEK(WORD, p + 6) <> 2 THEN pass = 0
    IF PEEK(WORD, p + 8) <> 3 THEN pass = 0
    ' DD &H12345678 at offsets 10..13 (LE)
    IF PEEK(DWORD, p + 10) <> &H12345678 THEN pass = 0
    ' DQ 1234567890 at offsets 14..21 (LE)
    IF PEEK(QUAD, p + 14) <> 1234567890 THEN pass = 0
    ' DB "Hi", 0 at offsets 22..24
    IF PEEK(BYTE, p + 22) <> 72 THEN pass = 0
    IF PEEK(BYTE, p + 23) <> 105 THEN pass = 0
    IF PEEK(BYTE, p + 24) <> 0 THEN pass = 0
    ' DW "AB" (WIDE, UTF-16LE) at offsets 25..28
    IF PEEK(WORD, p + 25) <> 65 THEN pass = 0
    IF PEEK(WORD, p + 27) <> 66 THEN pass = 0

    IF pass = 1 THEN
        ConPrint "Batch 30 ASMDATA: ALL PASS (14/14)"
    ELSE
        ConPrint "Batch 30 ASMDATA: FAIL"
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


