

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

' PB64Basic Enhanced - batch 45: ERL$ / EXTRACT$ / RGB / BGR
GLOBAL failures AS LONG

SUB Check(cond AS LONG, msg AS STRING)
    IF cond = 0 THEN
        ConPrint "FAIL: " & msg
        failures = failures + 1
    END IF
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL s AS STRING
    LOCAL n AS LONG
    failures = 0

    ' EXTRACT$: from position 1 to first match
    s = EXTRACT$("the cat sat on the mat", " sat")
    Check(s = "the cat", "extract basic")
    ' EXTRACT$ with start
    s = EXTRACT$(5, "the cat sat on the mat", " on")
    Check(s = "cat sat", "extract start")
    ' EXTRACT$ ANY: stop at any char of match set
    s = EXTRACT$("hello world", ANY " ")  ' first space
    Check(s = "hello", "extract space")
    ' EXTRACT$ ANY marker form
    s = EXTRACT$("ab,cd;ef", ANY ",;")  ' stop at first , or ;
    Check(s = "ab", "extract any")
    ' EXTRACT$ missing match -> whole remainder
    s = EXTRACT$("abcdef", "xyz")
    Check(s = "abcdef", "extract no match")
    ' EXTRACT$ start beyond length -> empty
    s = EXTRACT$(99, "abcdef", "x")
    Check(s = "", "extract beyond")

    ' RGB 3-arg: red + green<<8 + blue<<16
    n = RGB(255, 0, 0)
    Check(n = &HFF, "rgb red")
    n = RGB(0, 255, 0)
    Check(n = &H0000FF00, "rgb green")
    n = RGB(0, 0, 255)
    Check(n = &HFF0000, "rgb blue")
    n = RGB(1, 2, 3)
    Check(n = &H30201, "rgb mix")

    ' BGR 3-arg: swapped order
    n = BGR(255, 0, 0)
    Check(n = &HFF0000, "bgr red")
    n = BGR(1, 2, 3)
    Check(n = &H10203, "bgr mix")

    ' RGB 1-arg byte swap (BGR -> RGB)
    n = RGB(&H30201)
    Check(n = &H10203, "rgb swap")
    n = BGR(&HFF0000)
    Check(n = &HFF, "bgr swap")

    ' ERL$ inside the error handler: NAME of a nonexistent file raises the trap
    ' (checkpoint id recorded), ERL$ returns that id as a non-empty string.
    ON ERROR GOTO errHandler
    NAME "no_such_file_xyz.bas" AS "abc.bas"
    GOTO skipHandler
errHandler:
    s = ERL$
    Check(LEN(s) > 0, "erl$ non-empty")
    RESUME NEXT
skipHandler:
    ON ERROR GOTO 0

    IF failures = 0 THEN
        ConPrint "batch45: ALL PASS"
    ELSE
        ConPrint "batch45: FAILURES=" & STR$(failures)
    END IF
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





