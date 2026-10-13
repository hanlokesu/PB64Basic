

' --- PBWin10 stub branch: this sample exercises PowerBasilisk-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.



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
' PowerBasilisk Enhanced - batch 206 self-check - COMM failures are loud,
' and the channel goes where you wrote it
'---------------------------------------------------------------------
' TWO DEFECTS THIS PINS
'   1. `COMM OPEN` used to fail in complete silence.  The compiler discards the
'      runtime return value, so a serial port that does not exist produced no
'      error value the program could test and no message anywhere; later COMM
'      statements on that channel said nothing either.  Now every failure is
'      reported on standard error, at most once per channel, and a successful
'      open re-arms the message.
'   2. The channel is written `AS #1`.  Written the other way - `COMM OPEN
'      "COM1", 1` - the channel was never passed to codegen, which then read the
'      BAUD slot as the channel and opened the port on channel 0 while the rest
'      of the program talked to channel 1.  That form is now a compile error.
'
' WHAT TO WATCH FOR
'   The program runs to the end (a bad port is not fatal) and the warning goes
'   to standard error, not stdout:
'     Runtime error: COMM OPEN "COM404" failed on channel 1 - the port does not
'     exist, is already in use, or the name is wrong
'   "COM404" is deliberate: no machine has it, so the failure path is exercised
'   everywhere instead of depending on the hardware.
'
' WHY THE EXAMPLE CANNOT ASSERT THE MESSAGE
'   A PB program has no statement for reading another stream's stderr, so the
'   message itself is asserted by the probe that captures stderr; this file
'   asserts what a program CAN see - that the failure is survivable and that
'   later COMM statements survive too.
'
' Expected stdout:
'   survived a failed COMM OPEN written the correct way
'   survived COMM SEND / CLOSE / SET on a closed channel
'   === FAILURES: 0
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    fails = 0
    COMM OPEN "COM404" AS #1
    ConPrint "survived a failed COMM OPEN written the correct way"
    COMM SEND #1, "x"
    ConPrint "survived COMM SEND on a closed channel"
    COMM CLOSE #1
    ConPrint "survived COMM CLOSE on a closed channel"
    COMM SET #1, BAUD = 9600
    ConPrint "survived COMM SET on a closed channel"
    COMM RESET #1, FLOW
    IF fails = 0 THEN ConPrint "=== FAILURES: 0"
    FUNCTION = 0
' Press any key to exit...
ConWaitKey
END FUNCTION





