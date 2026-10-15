

#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- Dual-compiler self-check: this UDP loopback runs on BOTH compilers. ---
'     The UDP SEND numeric IP is interpreted differently by each side:
'     PBWin10 stores the number as-is (expects the NETWORK-order value
'     &H0100007F = 127.0.0.1 bytes 7F 00 00 01), while PB64Basic converts
'     the number host->network (expects the HOST-order value &H7F000001).
'     Passing the wrong value makes sendto a black-hole (1.0.0.127) and
'     RECV hangs forever - measured 2026-10-15 on both compilers.
'     A string "127.0.0.1" is rejected by PBWin10 (Error 420 - AT takes
'     ipaddr& only).  Hence the discriminator below.


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
' PB64Basic Enhanced - batch 203 self-check - UDP on loopback
'---------------------------------------------------------------------
' WHY THIS FILE EXISTS
'   Batch 201 and 202 could only say "the statement compiled and returned".
'   For a socket that is worth nothing: a bind that never binds looks exactly
'   like a bind that works when all you check is the return. This file is the
'   end-to-end proof, and it runs without any network card, router or peer
'   program - the socket talks to itself through 127.0.0.1.
'
' HOW IT PROVES THE SOCKET IS REAL
'   1. UDP OPEN PORT 46217 AS #1      - bind a real UDP socket
'   2. UDP SEND #1, AT "127.0.0.1", 46217, "self-ping"
'   3. UDP RECV #1, FROM ip, pnum, buf
'      The payload and the sender port can only appear if a datagram really
'      travelled through the operating system's socket layer.  A stubbed
'      pb_udp_recv would leave buf empty and the first check would fail.
'
' VERIFIED EXTERNALLY TOO
'   The same statements were driven from outside: a Python socket sent one
'   datagram to this port while the program sat inside UDP RECV, and the
'   program printed the foreign payload ("external-ping").  TCP was checked
'   the same way - `TCP OPEN SERVER PORT p AS #1` really listens, because an
'   external TCP client connected to it successfully.
'
' Expected output:
'   received: self-ping
'   from port: 46217
'   === FAILURES: 0
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL ip AS LONG
    LOCAL pnum AS LONG
    LOCAL buf AS STRING
    LOCAL fails AS LONG
    fails = 0
    UDP OPEN PORT 46217 AS #1
#IF %MY_PBVER = 10
    UDP SEND #1, AT &H0100007F, 46217, "self-ping"   ' PBWin: numeric stored as-is
#ELSE
    UDP SEND #1, AT &H7F000001, 46217, "self-ping"   ' fork: numeric host->network
#ENDIF
    UDP RECV #1, FROM ip, pnum, buf
    ConPrint "received: " & buf
    IF buf <> "self-ping" THEN fails = fails + 1
    ConPrint "from port: " & STR$(pnum)
    IF pnum <> 46217 THEN fails = fails + 1
    UDP CLOSE #1
    ConPrint "=== FAILURES: " & STR$(fails)
    FUNCTION = 0
' Press any key to exit...
ConWaitKey
END FUNCTION




