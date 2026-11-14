' PB64Basic Enhanced - batch 350 self-check - TCP LINE INPUT partial-line EOF
' CHW TCP_LINE_INPUT_statement.htm: if no CRLF has been found, TCP LINE
' returns whatever data has been accumulated and sets EOF to TRUE.
' This sample proves three things at run time:
'   1. a partial line ("partial", no CRLF) comes back from LINE INPUT;
'   2. EOF() is TRUE right after that partial line;
'   3. the next LINE INPUT consumes the remaining CRLF and returns an empty
'      complete line with EOF() FALSE.
' Expected output: partial=[partial] eof1=1 line2=[] eof2=0 === FAILURES: 0
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG
SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        IF AttachConsole(-1) = 0 THEN AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

GLOBAL g_srv_line AS STRING
GLOBAL g_srv_eof AS LONG
GLOBAL g_srv_line2 AS STRING
GLOBAL g_srv_eof2 AS LONG
GLOBAL g_accept_ready AS LONG
GLOBAL g_srv_done AS LONG

THREAD FUNCTION Srv(byval dummy AS LONG) AS LONG
    LOCAL s AS STRING
    LOCAL newf AS LONG
    g_accept_ready = 1
    TCP ACCEPT #1 AS newf
    TCP LINE INPUT newf, s
    g_srv_line = s
    IF EOF(newf) THEN g_srv_eof = 1
    TCP LINE INPUT newf, s
    g_srv_line2 = s
    IF EOF(newf) THEN g_srv_eof2 = 1
    g_srv_done = 1
    TCP CLOSE newf
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL th AS LONG
    LOCAL waits AS LONG
    fails = 0
    TCP OPEN SERVER ADDR &H0100007F PORT 46231 AS #1
    THREAD CREATE Srv(0) TO th
    waits = 0
    DO WHILE g_accept_ready = 0 AND waits < 100
        SLEEP 20
        waits = waits + 1
    LOOP
    TCP OPEN PORT 46231 AT "127.0.0.1" AS #3
    TCP SEND #3, "partial"
    SLEEP 200
    TCP SEND #3, CHR$(13) + CHR$(10)
    waits = 0
    DO WHILE g_srv_done = 0 AND waits < 200
        SLEEP 20
        waits = waits + 1
    LOOP
    ConPrint "partial=[" & g_srv_line & "] eof1=" & STR$(g_srv_eof) & " line2=[" & g_srv_line2 & "] eof2=" & STR$(g_srv_eof2)
    IF g_srv_line <> "partial" THEN fails = fails + 1
    IF g_srv_eof <> 1 THEN fails = fails + 1
    IF LEN(g_srv_line2) <> 0 THEN fails = fails + 1
    IF g_srv_eof2 <> 0 THEN fails = fails + 1
    THREAD CLOSE th
    TCP CLOSE #3
    TCP CLOSE #1
    ConPrint "=== FAILURES: " & STR$(fails)
    FUNCTION = 0
END FUNCTION