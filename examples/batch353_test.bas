#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 353 test
' TCP/UDP NOTIFY full CHW syntax, plus CHRBYTES on wide strings.
'
' This probe deliberately creates NO socket: binding a port makes Windows
' Firewall show its "allow this app" dialog, which a corpus gate must not do.
' Without a socket it still covers:
'   - the full NOTIFY clause parses and reaches the runtime with four arguments
'   - a file number that is not open is rejected (ERR <> 0)
'   - the legacy mask-only forms still compile
'   - CHRBYTES reports 2 for WSTRING and 1 for STRING (CHW CHRBYTES_function)
' The live WSAAsyncSelect registration is exercised by _net_probes/batch353_net.bas,
' which is run manually because it binds a port.
DECLARE FUNCTION PBMAIN() AS LONG

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG
    LOCAL fails AS LONG
    LOCAL ws AS WSTRING
    LOCAL ns AS STRING

    fails = 0
    ws = "wide"
    ns = "ansi"
    IF CHRBYTES(ws) <> 2 THEN
        PRINT "FAIL: CHRBYTES(WSTRING) ="; CHRBYTES(ws)
        fails = fails + 1
    END IF
    IF CHRBYTES(ns) <> 1 THEN
        PRINT "FAIL: CHRBYTES(STRING) ="; CHRBYTES(ns)
        fails = fails + 1
    END IF

    DIALOG NEW 0, "notify", 0, 0, 120, 80 TO hDlg
    IF hDlg = 0 THEN
        PRINT "FAIL: DIALOG NEW returned 0"
        fails = fails + 1
    END IF

    ERRCLEAR
    TCP NOTIFY #1, ACCEPT TO hDlg AS 1524
    PRINT "tcp notify closed file err="; ERR
    IF ERR = 0 THEN
        PRINT "FAIL: TCP NOTIFY on a closed file number set no error"
        fails = fails + 1
    END IF

    ERRCLEAR
    UDP NOTIFY #2, RECV TO hDlg AS 1527
    IF ERR = 0 THEN
        PRINT "FAIL: UDP NOTIFY on a closed file number set no error"
        fails = fails + 1
    END IF

    ' Legacy mask-only forms still compile (codegen keeps its warning).
    TCP NOTIFY #1, 3
    TCP NOTIFY #1, 1, 8

    PRINT "FAILURES: "; fails
    FUNCTION = 0
END FUNCTION
