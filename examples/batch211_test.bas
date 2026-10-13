#IF (%PB_REVISION AND &H0FF00) = &H1000
    ' Compiling with PB/Win 10.x
    %MY_PBVER = 10
#ELSEIF (%PB_REVISION AND &H0FF00) = &H0900
    ' Compiling with PB/Win 9.x
    %MY_PBVER = 9
#ELSE
    ' Not PBWin (this fork, or other)
    %MY_PBVER = 0
#ENDIF
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
' batch211_test.bas - batch 211: ACCEL ATTACH gets a real runtime
'---------------------------------------------------------------------
' ACCEL ATTACH was one of five statements that were parsed, compiled
' cleanly and produced no code at all (ACCEL_ATTACH / EVENT_SOURCE /
' EVENTS / RAISEEVENT / INSTANCE shared one empty arm).  Batch 211
' implements the first of them for real:
'
'   * the compiler builds the key table from the array's *declared*
'     element count - the source passes no count, because a compiler
'     knows the size of a constant array;
'   * the runtime calls CreateAcceleratorTableA and remembers
'     hDlg -> hAccel in a small registry;
'   * both message pumps (the modal GetMessage loop and
'     DIALOG DOEVENTS) offer every message to the attached tables with
'     TranslateAcceleratorA, which is what makes a key fire - a table
'     nobody consults would be the same no-op as before;
'   * the optional `TO hAccel` clause keeps the table handle.
'
' Documented record layout (one QUAD per accelerator).  The machine's
' official help carries no ACCEL_ATTACH page, so this layout is the
' fork's own documented shape, not a transcription of a PB page:
'
'     bits  0-15   fVirt   FVIRTKEY / FSHIFT / FCONTROL / FALT
'     bits 16-31   key     virtual key code
'     bits 32-47   cmd     command id (WM_COMMAND's low word)
'
' Why the records below carry no command id: this dialect evaluates
' `hi * 65536 * 65536` in 32 bits, where it wraps to 0 (measured - see
' the probe note in the skill), and a 16-hex-digit literal is the only
' other way to reach bits 32-47.  The record therefore holds fVirt+key
' only; composing the command field is an open item.
'
' Coverage, stated honestly:
'   * COVERED - a real table is created: the two-element array yields a
'     non-zero handle, and the TO target really receives a value (the
'     no-count case is checked against a value it cannot coincidentally
'     already hold).
'   * NOT COVERED HERE - TranslateAcceleratorA firing a key needs a
'     window to pump messages, which cannot be exercised without going
'     interactive; the pump hook is reached by the DDT dialog samples.
'
' The four remaining statements stay parked and reported: they need a
' CLASS / INTERFACE / METHOD object model that codegen does not have
' (0 hits for all three), and a no-op wearing an Implemented label is
' exactly what batch 210 had to undo.
'
' Asserts: 1. a two-element key array produces a non-zero table handle
'          2. an argument with no element count produces 0 (no table)
'             *and* really writes the TO target (started at 12345)
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL fail AS LONG
    LOCAL hDlg AS LONG
    LOCAL acc AS QUAD
    LOCAL none AS QUAD
    LOCAL keys() AS QUAD
    DIM keys(1) AS QUAD
    LOCAL plain AS QUAD

    ' FVIRTKEY|FCONTROL (0x0009) with 'A' (0x41) and with 'B' (0x42)
    keys(0) = &H00410009
    keys(1) = &H00420009
    ConPrint "keys(0) = " & STR$(keys(0))
    ConPrint "keys(1) = " & STR$(keys(1))

#IF %MY_PBVER = 0
    ACCEL ATTACH hDlg, keys() TO acc
#ELSE
' (PBWin10: skipped: ACCEL ATTACH hDlg, keys() TO acc)
#ENDIF
    ConPrint "accelerator table = " & STR$(acc)
    IF acc = 0 THEN
        fail = fail + 1
    END IF

    ' A non-array argument carries no element count, so no table can be
    ' built: the handle must come back 0 - and the TO target must still be
    ' written, which is why it starts at a value the program recognises.
    none = 12345
#IF %MY_PBVER = 0
    ACCEL ATTACH hDlg, plain TO none
#ELSE
' (PBWin10: skipped: ACCEL ATTACH hDlg, plain TO none)
#ENDIF
    ConPrint "no-count attach   = " & STR$(none)
    IF none <> 0 THEN
        fail = fail + 1
    END IF

    ConPrint "=== FAILURES:" & STR$(fail) & "==="
    FUNCTION = fail
' Press any key to exit...
    ConWaitKey
END FUNCTION
