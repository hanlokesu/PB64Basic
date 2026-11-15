#COMPILE EXE
#DIM ALL

' batch 357 self-check: TCP/UDP NOTIFY must deliver a REAL window message
' (WSAAsyncSelect -> WM_USER+n to the named hWnd).  The probe registers
' UDP RECV notification on a loopback socket, sends one datagram to itself
' and then Peeks the thread message queue for the exact wMsg, checking
' wParam (socket handle) and lParam low word (FD_READ).  A registration that
' only stored a mask would leave the queue empty.
'
' 32-bit variant: MSG uses 4-byte HWND/WPARAM/LPARAM.

TYPE B357MSG
    hwnd AS LONG
    message AS LONG
    wParam AS LONG
    lParam AS LONG
    tm AS LONG
    px AS LONG
    py AS LONG
END TYPE

DECLARE FUNCTION PeekMessageA LIB "USER32.DLL" ALIAS "PeekMessageA" (lpMsg AS B357MSG, BYVAL hWnd AS LONG, BYVAL wMsgFilterMin AS LONG, BYVAL wMsgFilterMax AS LONG, BYVAL wRemoveMsg AS LONG) AS LONG
DECLARE FUNCTION Sleep LIB "KERNEL32.DLL" ALIAS "Sleep" (BYVAL dwMilliseconds AS DWORD) AS LONG

FUNCTION PBMAIN () AS LONG
    LOCAL h AS LONG
    LOCAL i AS LONG
    LOCAL got AS LONG
    LOCAL evt AS LONG
    LOCAL wparam AS LONG
    LOCAL m AS B357MSG

    got = 0
    evt = 0
    wparam = 0

    WINDOW "b357 notify", 100, 100, 200, 100 TO h
    UDP OPEN PORT 46221 AS #1
    UDP NOTIFY #1, RECV TO h AS 1024
    UDP SEND #1, AT &H0100007F, 46221, "ping"

    FOR i = 1 TO 100
        IF PeekMessageA(m, 0, 0, 0, 1) THEN
            IF m.message = 1024 THEN
                got = 1
                wparam = m.wParam
                evt = m.lParam AND &HFFFF&
                EXIT FOR
            END IF
        END IF
        Sleep 20
    NEXT i

    PRINT "notify message="; m.message; " wParam="; wparam; " event="; evt; " got="; got
    PRINT " (want message=1024, event=1 FD_READ, got=1)"
    IF got = 0 THEN PRINT "FAIL: no WM_USER message was delivered to the window"
    IF got <> 0 AND evt <> 1 THEN PRINT "FAIL: lParam low word is not FD_READ"
    PRINT "=== FAILURES: "; (1 - got)

    DIALOG END h
    FUNCTION = 0
END FUNCTION
