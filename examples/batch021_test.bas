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

' batch21_test.bas - COMM serial port + THREAD statements (batch 21)
' worker uses a GLOBAL stop flag + SLEEP loop so it can exit before the
' process does - an infinite DO:LOOP worker made process exit hang.
GLOBAL g_stop AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL id AS LONG, st AS LONG, p AS LONG

    ConPrint "== COMM (no real COM port: expect graceful failure, no crash) =="
    COMM OPEN "COM1" AS #0, BAUD 9600, PARITY "N", DATA 8, STOP 1
    COMM RESET #0, FLOW
    ConPrint "COMM OK (no crash)"

    ConPrint "== THREAD =="
    THREAD CREATE worker TO id
    IF id < 0 THEN
        ConPrint "THREAD CREATE FAILED rc=" & STR$(id)
    ELSE
        ConPrint "thread id=" & STR$(id)
        THREAD STATUS id TO st
        ConPrint "status=" & STR$(st)
        THREAD SET PRIORITY id, 0
        THREAD GET PRIORITY id TO p
        ConPrint "priority=" & STR$(p)
        THREAD SUSPEND id
        THREAD STATUS id TO st
        ConPrint "after suspend status=" & STR$(st)
        THREAD RESUME id
        THREAD STATUS id TO st
        ConPrint "after resume status=" & STR$(st)
        g_stop = 1
        SLEEP 20
        THREAD CLOSE id
        ConPrint "THREAD OK"
    END IF

    WAITKEY$
END FUNCTION

SUB worker()
    ' SLEEP is a call boundary: keeps the g_stop load fresh (see batch028 NOTE)
    DO WHILE g_stop = 0
        SLEEP 1
    LOOP
END SUB


#ENDIF
