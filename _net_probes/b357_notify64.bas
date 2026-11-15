#COMPILE EXE
#DIM ALL

' batch 357 self-check (64-bit variant): UDP NOTIFY must post a REAL
' WM_USER+ message to the named window.  MSG is the native 64-bit layout
' (HWND/WPARAM/LPARAM are 8 bytes, message/time are 4 bytes with padding).

TYPE B357MSG64
    hwnd AS QUAD
    message AS LONG
    pad1 AS LONG
    wParam AS QUAD
    lParam AS QUAD
    tm AS LONG
    pad2 AS LONG
    px AS LONG
    py AS LONG
END TYPE

DECLARE FUNCTION PeekMessageA LIB "USER32.DLL" ALIAS "PeekMessageA" (lpMsg AS B357MSG64, BYVAL hWnd AS QUAD, BYVAL wMsgFilterMin AS LONG, BYVAL wMsgFilterMax AS LONG, BYVAL wRemoveMsg AS LONG) AS LONG
DECLARE FUNCTION Sleep LIB "KERNEL32.DLL" ALIAS "Sleep" (BYVAL dwMilliseconds AS DWORD) AS LONG

FUNCTION PBMAIN () AS LONG
    LOCAL h AS QUAD
    LOCAL i AS LONG
    LOCAL got AS LONG
    LOCAL evt AS LONG
    LOCAL wparam AS QUAD
    LOCAL m AS B357MSG64

    got = 0
    evt = 0
    wparam = 0

    WINDOW "b357 notify64", 100, 100, 200, 100 TO h
    UDP OPEN PORT 46222 AS #1
    UDP NOTIFY #1, RECV TO h AS 1025
    UDP SEND #1, AT &H0100007F, 46222, "ping"

    FOR i = 1 TO 100
        IF PeekMessageA(m, 0, 0, 0, 1) THEN
            IF m.message = 1025 THEN
                got = 1
                wparam = m.wParam
                evt = m.lParam AND &HFFFF&
                EXIT FOR
            END IF
        END IF
        Sleep 20
    NEXT i

    PRINT "notify message="; m.message; " wParam="; wparam; " event="; evt; " got="; got
    PRINT " (want message=1025, event=1 FD_READ, got=1)"
    IF got = 0 THEN PRINT "FAIL: no WM_USER message was delivered to the window"
    IF got <> 0 AND evt <> 1 THEN PRINT "FAIL: lParam low word is not FD_READ"
    PRINT "=== FAILURES: "; (1 - got)

    DIALOG END h
    FUNCTION = 0
END FUNCTION
