#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 353 manual network check (NOT part of the gates).
'
' TCP/UDP NOTIFY registers the socket with WSAAsyncSelect.  Proving that
' registration succeeds needs a live socket, and a live socket needs a bound
' port - which makes Windows Firewall show its "allow this app" dialog on the
' first run.  The automated corpus gate therefore uses examples/batch353_test.bas,
' which creates no socket.  Run THIS file by hand when you want the end-to-end
' check (allow the firewall prompt once, or run it on a machine whose policy
' already allows the binary).
'
' Expected output:
'   tcp accept err= 0
'   bad hwnd err= 57
'   udp recv err= 0
'   FAILURES:  0
DECLARE FUNCTION PBMAIN() AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG
    LOCAL fails AS LONG

    fails = 0
    DIALOG NEW 0, "notify", 0, 0, 120, 80 TO hDlg
    IF hDlg = 0 THEN
        PRINT "FAIL: DIALOG NEW returned 0"
        fails = fails + 1
    END IF

    ' TCP server on loopback: bind + listen, then register FD_ACCEPT.
    TCP OPEN SERVER ADDR &H0100007F PORT 46353 AS #1
    IF ERR <> 0 THEN
        PRINT "FAIL: TCP OPEN SERVER ERR="; ERR
        fails = fails + 1
    END IF

    ERRCLEAR
    TCP NOTIFY #1, ACCEPT TO hDlg AS 1524
    PRINT "tcp accept err="; ERR
    IF ERR <> 0 THEN
        PRINT "FAIL: TCP NOTIFY ACCEPT ERR="; ERR
        fails = fails + 1
    END IF

    ' A non-window handle makes WSAAsyncSelect fail: Error 57.
    ERRCLEAR
    TCP NOTIFY #1, ACCEPT TO 12345 AS 1525
    PRINT "bad hwnd err="; ERR
    IF ERR <> 57 THEN
        PRINT "FAIL: invalid hwnd did not raise Error 57 (ERR="; ERR; ")"
        fails = fails + 1
    END IF

    ' Re-registering the real window must succeed again.
    ERRCLEAR
    TCP NOTIFY #1, ACCEPT TO hDlg AS 1526
    IF ERR <> 0 THEN
        PRINT "FAIL: re-register ERR="; ERR
        fails = fails + 1
    END IF

    ' UDP socket bound to a fixed port, registered for RECV and CLOSE.
    UDP OPEN PORT 46354 AS #2
    IF ERR <> 0 THEN
        PRINT "FAIL: UDP OPEN ERR="; ERR
        fails = fails + 1
    END IF

    ERRCLEAR
    UDP NOTIFY #2, RECV TO hDlg AS 1527
    PRINT "udp recv err="; ERR
    IF ERR <> 0 THEN
        PRINT "FAIL: UDP NOTIFY RECV ERR="; ERR
        fails = fails + 1
    END IF

    ERRCLEAR
    UDP NOTIFY #2, CLOSE TO hDlg AS 1528
    IF ERR <> 0 THEN
        PRINT "FAIL: UDP NOTIFY CLOSE ERR="; ERR
        fails = fails + 1
    END IF

    PRINT "FAILURES: "; fails
    FUNCTION = 0
END FUNCTION
