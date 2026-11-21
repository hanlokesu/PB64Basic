#COMPILE EXE
#DIM ALL
' Regression: WSAAsyncSelect notifications must reach the DIALOG callback.
' A TCP NOTIFY ACCEPT event is posted as WM_USER+1024; the runtime WndProc
' has to forward it (CB.MSG=1024, CB.WPARAM=socket, CB.LPARAM=FD_ACCEPT).
GLOBAL hDlg AS LONG
GLOBAL notifyCount AS LONG

FUNCTION PBMAIN () AS LONG
    LOCAL rc AS LONG
    LOCAL nf AS LONG
    LOCAL t0 AS SINGLE
    TCP OPEN SERVER ADDR &H0100007F PORT 46291 AS #1
    rc = ERR
    DIALOG NEW 0, "b401 tcp notify", 100, 100, 300, 150 TO hDlg
    TCP NOTIFY 1, ACCEPT TO hDlg AS 1024
    OPEN "batch401_tcp_notify.txt" FOR OUTPUT AS #2
    PRINT #2, "open_rc=" & STR$(rc) & " notify_err=" & STR$(ERR)
    CLOSE #2
    DIALOG SHOW MODELESS hDlg CALL DlgProc
    t0 = TIMER
    DO
        DIALOG DOEVENTS
        SLEEP 10
    LOOP WHILE notifyCount = 0 AND TIMER - t0 < 8
    OPEN "batch401_tcp_notify.txt" FOR APPEND AS #2
    PRINT #2, "count=" & STR$(notifyCount)
    CLOSE #2
    DIALOG END hDlg
END FUNCTION

CALLBACK FUNCTION DlgProc()
    LOCAL nf AS LONG
    IF CB.MSG = 1024 THEN
        notifyCount = notifyCount + 1
        TCP ACCEPT 1 AS nf
        OPEN "batch401_tcp_notify.txt" FOR APPEND AS #2
        PRINT #2, "msg=" & STR$(CB.MSG) & " wparam=" & STR$(CB.WPARAM) & " lparam=" & STR$(CB.LPARAM)
        PRINT #2, "accept=" & STR$(nf) & " err=" & STR$(ERR)
        CLOSE #2
        DIALOG END hDlg
    END IF
END FUNCTION
