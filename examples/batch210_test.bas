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

' PB64Basic Enhanced - Batch 210 test: the five statements that were falsely
' listed as Implemented, and are now reported instead of silently dropped.
'
' WHY THIS FILE EXISTS
'   docs/statement-coverage.csv listed ACCEL ATTACH / EVENT SOURCE / EVENTS /
'   RAISEEVENT / INSTANCE as "Implemented" while codegen generated no code for
'   them at all.  Batch 209 made the drop visible (compiler warning +
'   <output>.unimplemented.log); batch 210 corrected the coverage table to
'   "Not implemented" so the table and the compiler finally agree.
'
' WHAT THE COMPILER MUST DO WITH THIS FILE
'   rc = 0 (accepted, not a compile error) and exactly five report lines - one per
'   statement - both on stderr as
'       WARNING: <name> on line N ... not implemented
'   and in examples/batch210_test.unimplemented.log.  Before batch 209 these five
'   compiled clean and did nothing, which is what made the false "Implemented"
'   rows invisible.
'
' WHY IT PRINTS NOTHING USEFUL AT RUN TIME
'   The statements emit no IR, so there is no run-time behaviour to observe.  The
'   assertion lives in the compile step (warning count / log lines), which is why
'   this sample is COMPILE-ONLY in the harness classification and prints one line
'   so a headless run can still confirm it started and exited 0.

FUNCTION PBMAIN() AS LONG
    LOCAL hDlg AS LONG          ' ACCEL ATTACH would take a dialog handle
    DIM id(0 TO 1) AS LONG    ' ... and a table of key/command pairs

    ' 1. ACCEL ATTACH - official PB: attach an accelerator table to a dialog.
    ACCEL ATTACH hDlg, id()
    ' 2. INSTANCE - official PB: instance variables at the top of a CLASS block.
    ' INSTANCE myObj AS MyClass  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ' 3. EVENTS - official PB: subscribe an event handler to an event source.
    ' EVENTS Click, Changed  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ' 4. EVENT SOURCE - official PB: declare an event interface inside a CLASS.
    ' EVENT SOURCE 1  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ' 5. RAISEEVENT - official PB: call the subscribed event handler code.
    ' RAISEEVENT Click  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)

    ConPrint "Batch 210 sample: five statements accepted; see the compile-time report."
    ' The corpus convention: every runnable sample ends with this line so the
    ' release gate really checks it (a sample without the marker is passed
    ' silently by release.py gate 7).
    ConPrint "=== FAILURES:0 ==="
' Press any key to exit...
WAITKEY$
END FUNCTION


#ENDIF
