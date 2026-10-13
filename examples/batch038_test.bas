

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

' PowerBasilisk Enhanced - batch 38: TALLY / STRREVERSE$ / STRINSERT$ / STRDELETE$ / REPEAT$ / FRAC / ISFOLDER / EXP2 / EXP10 / LOG2 / LOG10 / IIF / CHOOSE
GLOBAL failures AS LONG

SUB Check(cond AS LONG, msg AS STRING)
    IF cond = 0 THEN
        ConPrint "FAIL: " & msg
        failures = failures + 1
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL x AS STRING
    LOCAL n AS LONG
    LOCAL d AS DOUBLE
    LOCAL r AS STRING
    LOCAL i AS LONG
    failures = 0

    ' TALLY
    n = TALLY("the cat sat on the mat", "the")
    Check(n = 2, "tally")
    n = TALLY("abc", "z")
    Check(n = 0, "tally none")

    ' STRREVERSE$
    r = STRREVERSE$("PowerBASIC")
    Check(r = "CISABrewoP", "strreverse")

    ' STRINSERT$ (1-based)
    r = STRINSERT$("abcd", "XY", 3)
    Check(r = "abXYcd", "strinsert")
    r = STRINSERT$("ab", "XY", 99)
    Check(r = "abXY", "strinsert append")

    ' STRDELETE$
    r = STRDELETE$("PowerBASIC", 4, 2)
    Check(r = "PowBASIC", "strdelete")
    r = STRDELETE$("abc", 2, 5)
    Check(r = "a", "strdelete overflow")

    ' REPEAT$
    r = REPEAT$(3, "ab")
    Check(r = "ababab", "repeat")

    ' FRAC
    d = FRAC(10.25)
    Check(d = 0.25, "frac")
    d = FRAC(-10.25)
    Check(d = -0.25, "frac neg")

    ' ISFOLDER
    n = ISFOLDER(".")
    Check(n = -1, "isfolder dot")
    n = ISFOLDER("C:\Windows")
    Check(n = -1, "isfolder windows")
    n = ISFOLDER("Z:\no_such_dir_xyz")
    Check(n = 0, "isfolder missing")

    ' EXP2 / EXP10 / LOG2 / LOG10
    d = EXP2(3)
    Check(d = 8, "exp2")
    d = EXP10(2)
    Check(d = 100, "exp10")
    d = LOG2(8)
    Check(d = 3, "log2")
    d = LOG10(1000)
    Check(d = 3, "log10")

    ' IIF numeric / string
    i = IIF(1, 10, 20)
    Check(i = 10, "iif true")
    i = IIF(0, 10, 20)
    Check(i = 20, "iif false")
    r = IIF$(1, "yes", "no")
    Check(r = "yes", "iif str true")
    r = IIF$(0, "yes", "no")
    Check(r = "no", "iif str false")

    ' CHOOSE
    i = CHOOSE(2, 10, 20, 30)
    Check(i = 20, "choose 2")
    i = CHOOSE(1, 10, 20, 30)
    Check(i = 10, "choose 1")
    i = CHOOSE(9, 10, 20, 30)
    Check(i = 10, "choose out of range")
    r = CHOOSE$(3, "a", "b", "c")
    Check(r = "c", "choose str")

    IF failures = 0 THEN
        ConPrint "batch38: ALL PASS"
    ELSE
        ConPrint "batch38: FAILURES=" & STR$(failures)
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





