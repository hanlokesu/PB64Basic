#COMPILE EXE
#DIM ALL


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

' PB64Basic Enhanced - batch 28: THREADED (thread-local storage)
' THREADED vars are global to every Sub/Function but each thread has its
' own independent copy. We verify: main sets tcount=1, a child thread sets
' its own tcount=2 and copies it to a plain global; main still reads 1.
'
' NOTE: the main thread waits with SLEEP, not a busy loop -- an optimizer
' is allowed to hoist an invariant global load out of a busy loop, so a
' DO WHILE flag=0 spin can keep reading the pre-thread value forever.


GLOBAL worker_ran AS LONG
GLOBAL worker_val AS LONG
GLOBAL worker_msg AS STRING

THREADED tcount AS LONG
THREADED tmsg AS STRING

THREAD FUNCTION Worker(byval dummy AS LONG) AS LONG
    tcount = 2
    tmsg = "worker"
    worker_val = tcount
    worker_msg = tmsg
    worker_ran = 1
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL lres AS LONG
    LOCAL th AS LONG

    tcount = 1
    tmsg = "main"

    THREAD CREATE Worker(0) TO th

    ' Give the child thread time to run (SLEEP is a call boundary, so the
    ' loads below are fresh; THREAD CLOSE itself does not wait).
    SLEEP 500

    IF worker_ran = 0 THEN
        ConPrint "worker did NOT run -> THREAD CREATE issue"
    ELSE
        ConPrint "worker ran: worker_val=" & STR$(worker_val) & " worker_msg=" & worker_msg
    END IF

    IF tcount = 1 AND tmsg = "main" THEN
        ConPrint "main still sees tcount=1 tmsg=main  -> TLS OK"
    ELSE
        ConPrint "main sees tcount=" & STR$(tcount) & " tmsg=" & tmsg & " -> TLS FAIL"
    END IF

    THREAD CLOSE th TO lres
    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION





