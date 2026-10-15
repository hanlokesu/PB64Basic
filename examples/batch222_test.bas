#COMPILE EXE
#DIM ALL
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

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

' batch222_test.bas — OOP event bus (EVENT SOURCE / EVENTS FROM / RAISEEVENT)
' ============================================================================
' Demonstrates:
'   * EVENT SOURCE Iface inside a CLASS — advertises an event interface
'   * RAISEEVENT Iface.Method(args) inside a method — fires an event
'   * EVENTS FROM obj / EVENTS END obj — subscribe / unsubscribe a client
'
' The event bus (batch 222) matches handlers by METHOD NAME: a subscribing
' class must implement a method with the same name as the raised event.
'
' Expected output (console):
'   got=42
'   (exit code 0)
' ============================================================================


CLASS Publisher
    INSTANCE state AS LONG
    EVENT SOURCE Status
    METHOD SetState (BYVAL v AS LONG)
        state = v
    END METHOD
    METHOD RaiseProgress (BYVAL pct AS LONG)
        RAISEEVENT Status.Progress(pct)
    END METHOD
END CLASS

CLASS Subscriber
    INSTANCE got AS LONG
    METHOD Progress (BYVAL pct AS LONG)
        got = pct
    END METHOD
END CLASS

FUNCTION PBMAIN () AS LONG
    LOCAL p AS Publisher
    LOCAL s AS Subscriber

    EVENTS FROM s
    p.SetState(5)
    p.RaiseProgress(42)
    ConPrint "got=" & STR$(s.got)
    WAITKEY$
    EVENTS END s

    FUNCTION = 0
END FUNCTION


#ENDIF
