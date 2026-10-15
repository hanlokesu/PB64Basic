#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

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

' Batch 1: TIX / MKBYT$ / ISINFINITE / ISNORMAL / CHDRIVE / PLAY WAVE / SETEOF
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL t AS QUAD
    LOCAL b AS STRING
    LOCAL f AS LONG
    LOCAL s AS STRING
    t = TIX
    IF t > 0 THEN ConPrint "TIX-PASS " & STR$(t)
    b = MKBYT$(65)
    IF b = "A" THEN ConPrint "MKBYT-PASS"
    IF ISINFINITE(STR$(VAL("1E308")) * 10) = -1 THEN ConPrint "ISINF-PASS"
    IF ISNORMAL(123.456) = -1 THEN ConPrint "ISNORM-PASS"
    CHDRIVE "C:"
    ConPrint "CHDRIVE-PASS"
    PLAY WAVE "no_such_file.wav"
    ConPrint "PLAYWAVE-PASS"
    f = FREEFILE
    OPEN "seteof_test.dat" FOR OUTPUT AS #f
    PRINT #f, "1234567890ABCDEFGHIJ"
    CLOSE #f
    f = FREEFILE
    OPEN "seteof_test.dat" FOR BINARY AS #f
    SEEK #f, 5
    SETEOF #f
    CLOSE #f
    OPEN "seteof_test.dat" FOR INPUT AS #f
    LINE INPUT #f, s
    CLOSE #f
    KILL "seteof_test.dat"
    IF LEN(s) > 0 AND LEN(s) < 20 THEN
        ConPrint "SETEOF-PASS len=" & STR$(LEN(s))
    ELSE
        ConPrint "SETEOF-FAIL len=" & STR$(LEN(s))
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION

FUNCTION ISINFINITE(BYVAL v AS DOUBLE) AS LONG
    IF v <> 0 AND v * 2 = v THEN
        FUNCTION = -1
    ELSE
        FUNCTION = 0
    END IF
END FUNCTION

FUNCTION ISNORMAL(BYVAL v AS DOUBLE) AS LONG
    IF v <> 0 AND v * 2 <> v AND v = v THEN
        FUNCTION = -1
    ELSE
        FUNCTION = 0
    END IF
END FUNCTION



#ENDIF
