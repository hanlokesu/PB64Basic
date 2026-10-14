

' --- Dual-compiler self-check: this sample runs on BOTH compilers. ---
'     Measured 2026-10-14: on this machine PBWin10's TCP OPEN SERVER does
'     not actually listen (external connect gets WinError 10061, main-thread
'     TCP ACCEPT blocks forever), so the PBWin side reports FAILURES 3.
'     That is a PBWin10 environment behaviour, not a fork defect - the fork
'     side passes with FAILURES 0 (TCP ACCEPT output-file-number fix,
'     batch 205).  The expected output below is the fork's verified output.



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
' PowerBasilisk Enhanced - batch 205 self-check - TCP loopback in ONE program
'---------------------------------------------------------------------
' WHY BATCH 204 COULD NOT SHIP THIS
'   Batch 204 sent bare bytes - `TCP SEND #3, "ping"` and `TCP SEND #2, "pong"` -
'   and then called `TCP LINE INPUT` on both ends.  LINE INPUT reads until a
'   line terminator, so each side waited for a newline the other never sent:
'   a mutual deadlock.  That was my test's fault, not the runtime's - the
'   runtime's `pb_tcp_open` / `pb_tcp_accept` are plain blocking calls with no
'   lock, and batch 204's external-peer probes passed because the Python peer
'   sent its line with a trailing newline.
'
' WHAT THIS FILE DOES DIFFERENTLY
'   It uses TCP PRINT, which appends CR/LF (`pb_tcp_print(f, data, nl)`), so the
'   conversation is line-oriented on both sides - the way a line-oriented
'   protocol must be written.
'
' THE CONVERSATION
'   main   : TCP OPEN SERVER PORT 46219 AS #1
'   thread : TCP ACCEPT #1 AS #2  ->  TCP LINE INPUT #2, s  ->  TCP PRINT #2, "pong"
'   main   : TCP OPEN PORT 46219 AT "127.0.0.1" AS #3
'            TCP PRINT #3, "ping"  ->  TCP LINE INPUT #3, s
'   "pong" can only reach the client if accept, send and line input all worked.
'
' Expected output:
'   client got: pong
'   server read: ping
'   accepted flag: 1
'   === FAILURES: 0
'=====================================================================
GLOBAL g_accepted AS LONG
GLOBAL g_server_line AS STRING

THREAD FUNCTION Acceptor(byval dummy AS LONG) AS LONG
    LOCAL s AS STRING
    LOCAL newf AS LONG
    TCP ACCEPT #1 AS newf
    g_accepted = 1
    TCP LINE INPUT newf, s
    g_server_line = s
    TCP PRINT newf, "pong"
    TCP CLOSE newf
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL s AS STRING
    LOCAL newf AS LONG
    LOCAL fails AS LONG
    LOCAL th AS LONG
    fails = 0
    TCP OPEN SERVER PORT 46219 AS #1
    THREAD CREATE Acceptor(0) TO th
    SLEEP 400
    TCP OPEN PORT 46219 AT "127.0.0.1" AS #3
    TCP PRINT #3, "ping"
    TCP LINE INPUT #3, s
    ConPrint "client got: " & s
    IF s <> "pong" THEN fails = fails + 1
    SLEEP 500
    ConPrint "server read: " & g_server_line
    IF g_server_line <> "ping" THEN fails = fails + 1
    ConPrint "accepted flag: " & STR$(g_accepted)
    IF g_accepted <> 1 THEN fails = fails + 1
    TCP CLOSE #3
    TCP CLOSE #1
    ConPrint "=== FAILURES: " & STR$(fails)
    FUNCTION = 0
' Press any key to exit...
ConWaitKey
END FUNCTION





