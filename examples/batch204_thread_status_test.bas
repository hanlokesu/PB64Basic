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

'=====================================================================
' PB64Basic Enhanced - batch 204 self-check - THREAD STATUS / CLOSE
'---------------------------------------------------------------------
' WHAT WAS MEASURED BEFORE THIS FILE WAS WRITTEN
'   Worker sleeps 700 ms, main checks at 150 ms and again at 1350 ms:
'       status while running: 1
'       status after finish:  3
'       flag: 1
'       close returned
'   So 1 means "alive" and 3 means "finished" in this runtime, and the handle
'   THREAD CREATE ... TO h hands back is a zero-based slot index (the first
'   thread is 0, which is a valid handle - never test a handle against 0).
'   THREAD CLOSE on a finished thread returns without complaint.
'
' WHY IT IS WORTH A SELF-CHECK
'   `THREAD STATUS h TO st` is the only way a program can learn that a worker
'   finished; if the status code were ever left at a stale value the loop that
'   waits for a thread would spin forever.  This file pins the two codes.
'
' NOTE ON THE TCP SIDE (batch 204, verified but not asserted here)
'   TCP was proven with an external peer on each side: PB as client sent a line
'   that a Python server received; PB as server (accept in the main thread AND
'   accept inside a THREAD) read a line a Python client sent.  A single program
'   playing both ends deadlocked, so no such example ships - the reason is not
'   yet understood and is recorded as an open item rather than papered over.
'
' Expected output:
'   status while running: 1
'   status after finish: 3
'   flag: 1
'   closes cleanly
'   === FAILURES: 0
'=====================================================================
GLOBAL g_flag AS LONG

FUNCTION Worker () AS LONG
    SLEEP 700
    g_flag = 1
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL st AS LONG
    LOCAL h AS QUAD
    LOCAL fails AS LONG
    fails = 0
    THREAD CREATE Worker TO h
    SLEEP 150
    THREAD STATUS h TO st
    ConPrint "status while running: " & STR$(st)
    IF st <> 1 THEN fails = fails + 1
    SLEEP 1200
    THREAD STATUS h TO st
    ConPrint "status after finish: " & STR$(st)
    IF st <> 3 THEN fails = fails + 1
    ConPrint "flag: " & STR$(g_flag)
    IF g_flag <> 1 THEN fails = fails + 1
    THREAD CLOSE h
    ConPrint "closes cleanly"
    ConPrint "=== FAILURES: " & STR$(fails)
    FUNCTION = 0
' Press any key to exit...
WAITKEY$
END FUNCTION


#ENDIF
