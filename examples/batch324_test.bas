#COMPILE EXE

' batch 324 self-check: TCP CHW conformance fixes.
'   1) TCP SEND is NUL-safe: a 5-byte string with embedded NUL bytes must be
'      delivered in full (a strlen-based send truncates at the first NUL).
'   2) A timed-out TCP RECV returns Error 24 (Device timeout) and an empty
'      result string. The 60 s default is verified separately; here an
'      explicit 800 ms timeout keeps the headless run fast.
' All traffic is over 127.0.0.1 loopback (no external connection, no firewall).

DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
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

GLOBAL g_binReady AS LONG
GLOBAL g_bin AS STRING
GLOBAL g_holdReady AS LONG
GLOBAL g_holdGo AS LONG

THREAD FUNCTION SrvBin(BYVAL d AS LONG) AS LONG
    LOCAL nf AS LONG
    LOCAL buf AS STRING
    TCP ACCEPT #1 AS nf
    g_bin = ""
    DO
        TCP RECV nf, 5 - LEN(g_bin), buf
        g_bin = g_bin + buf
        IF LEN(g_bin) >= 5 THEN EXIT DO
        IF LEN(buf) = 0 THEN EXIT DO
    LOOP
    g_binReady = 1
    FUNCTION = 0
END FUNCTION

THREAD FUNCTION SrvHold(BYVAL d AS LONG) AS LONG
    LOCAL nf AS LONG
    TCP ACCEPT #2 AS nf
    g_holdReady = 1
    WHILE g_holdGo
        SLEEP 100
    WEND
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL th1 AS LONG, th2 AS LONG
    LOCAL s AS STRING, buf AS STRING
    LOCAL n AS LONG, e AS LONG, fails AS LONG
    fails = 0
    g_holdGo = 1
    g_binReady = 0
    g_holdReady = 0
    TCP OPEN SERVER ADDR &H0100007F PORT 46271 AS #1
    TCP OPEN SERVER ADDR &H0100007F PORT 46272 AS #2
    THREAD CREATE SrvBin(0) TO th1
    THREAD CREATE SrvHold(0) TO th2

    ' --- phase 1: binary NUL-safe SEND ---
    TCP OPEN PORT 46271 AT "127.0.0.1" AS #10
    s = CHR$(65) + CHR$(0) + CHR$(66) + CHR$(0) + CHR$(67)
    TCP SEND #10, s
    n = 0
    WHILE g_binReady = 0 AND n < 100
        SLEEP 50
        n = n + 1
    WEND
    ConPrint "binary gotLen: " & STR$(LEN(g_bin))
    IF LEN(g_bin) <> 5 THEN fails = fails + 1
    IF ASC(g_bin, 1) <> 65 THEN fails = fails + 1
    IF ASC(g_bin, 2) <> 0 THEN fails = fails + 1
    IF ASC(g_bin, 3) <> 66 THEN fails = fails + 1
    IF ASC(g_bin, 4) <> 0 THEN fails = fails + 1
    IF ASC(g_bin, 5) <> 67 THEN fails = fails + 1
    TCP CLOSE #10

    ' --- phase 2: timeout Error 24 ---
    TCP OPEN PORT 46272 AT "127.0.0.1" AS #11 TIMEOUT 800
    n = 0
    WHILE g_holdReady = 0 AND n < 100
        SLEEP 50
        n = n + 1
    WEND
    ERRCLEAR
    TCP RECV #11, 16, buf
    e = ERR
    ConPrint "timeout recvLen: " & STR$(LEN(buf)) & " err: " & STR$(e)
    IF e <> 24 THEN fails = fails + 1
    IF LEN(buf) <> 0 THEN fails = fails + 1
    TCP CLOSE #11

    g_holdGo = 0
    SLEEP 250
    TCP CLOSE #1
    TCP CLOSE #2
    ConPrint "=== FAILURES: " & STR$(fails)
    FUNCTION = 0
END FUNCTION
