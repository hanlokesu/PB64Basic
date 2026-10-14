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

' PowerBasilisk Enhanced - batch 39: BIN$ / OCT$ / DEC$ / VERIFY / MOD / GETATTR / DISKFREE / DISKSIZE
GLOBAL failures AS LONG

SUB Check(cond AS LONG, msg AS STRING)
    IF cond = 0 THEN
        ConPrint "FAIL: " & msg
        failures = failures + 1
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL r AS STRING
    LOCAL n AS LONG
    LOCAL q AS QUAD
    LOCAL f AS LONG
    failures = 0

    ' BIN$
    r = BIN$(5)
    Check(r = "101", "bin 5")
    r = BIN$(0)
    Check(r = "0", "bin 0")
    r = BIN$(13)
    Check(r = "1101", "bin 13")

    ' OCT$
    r = OCT$(8)
    Check(r = "10", "oct 8")
    r = OCT$(64)
    Check(r = "100", "oct 64")

    ' DEC$
    r = DEC$(255)
    Check(r = "255", "dec 255")
    r = DEC$(-7)
    Check(r = "-7", "dec neg")

    ' VERIFY
    n = VERIFY("abc", "xyz")
    Check(n = 1, "verify first")
    n = VERIFY("abc", "abcxyz")
    Check(n = 0, "verify all match")
    n = VERIFY(2, "abc", "ab")
    Check(n = 3, "verify from 2")

    ' MOD (truncated remainder, same as C srem)
    n = 10 MOD 3
    Check(n = 1, "mod 10 3")
    n = 13 MOD 5
    Check(n = 3, "mod 13 5")
    n = -7 MOD 3
    Check(n = -1, "mod neg")

    ' GETATTR
    f = GETATTR(".")
    Check(f <> 0, "getattr dot exists")
    f = GETATTR("Z:\no_such_file_xyz_123")
    Check(f = 0, "getattr missing")

    ' DISKFREE / DISKSIZE (bytes)
    q = DISKFREE("C:\")
    Check(q > 0, "diskfree c")
    q = DISKSIZE("C:\")
    Check(q > 0, "disksize c")
    q = DISKFREE("")
    Check(q > 0, "diskfree default")

    IF failures = 0 THEN
        ConPrint "batch39: ALL PASS"
    ELSE
        ConPrint "batch39: FAILURES=" & STR$(failures)
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


