#COMPILE EXE
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

' batch22_test.bas - LPRINT / TRACE / IMPORT / CALL DWORD (batch 22)

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL memaddr&, hndl&, t&
    LOCAL s AS STRING

    ' --- LPRINT: attach to a file (device), print, formfeed, flush, close ---
    LPRINT ATTACH "lprint_test.out"
    LPRINT "Hello LPRINT"
    LPRINT "num="; 123
    LPRINT "pi="; 3.14
    LPRINT FORMFEED
    LPRINT FLUSH
    LPRINT CLOSE
    s = ""
    OPEN "lprint_test.out" FOR INPUT AS #1
    LINE INPUT #1, s
    CLOSE #1
    ConPrint "lprint line1 = " & s
    IF INSTR(s, "Hello LPRINT") > 0 THEN
        ConPrint "LPRINT OK"
    ELSE
        ConPrint "LPRINT FAIL"
    END IF

    ' --- TRACE: explicit trace file ---
    TRACE NEW "trace_test.log"
    TRACE ON
    TRACE PRINT "marker-1"
    TRACE OFF
    TRACE PRINT "should-not-appear"
    TRACE CLOSE
    s = ""
    OPEN "trace_test.log" FOR INPUT AS #1
    LINE INPUT #1, s
    CLOSE #1
    ConPrint "trace line1 = " & s
    IF INSTR(s, "marker-1") > 0 THEN
        ConPrint "TRACE OK"
    ELSE
        ConPrint "TRACE FAIL"
    END IF

    ' --- IMPORT ADDR + CALL DWORD (GetTickCount) ---
    IMPORT ADDR "GetTickCount", "KERNEL32.DLL" TO memaddr&, hndl&
    IF memaddr& <> 0 THEN
        CALL DWORD memaddr& USING GetTickCount() TO t&
        ConPrint "tick = " & STR$(t&)
        IF t& > 0 THEN
            ConPrint "CALL DWORD OK"
        ELSE
            ConPrint "CALL DWORD FAIL"
        END IF
        IMPORT CLOSE hndl&
    ELSE
        ConPrint "IMPORT ADDR FAIL"
    END IF

    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
