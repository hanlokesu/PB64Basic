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

' PB64Basic Enhanced - Batch 196 test: one warning per statement, not two
'
' The four statements below are the group that batch 195 started reporting: the empty
' codegen arm shared by INSTANCE / EVENTS / EVENT SOURCE / RAISEEVENT / ACCEL_ATTACH
' and the "known family, no arm of its own" early return.  Batch 195 pushed a warning
' from BOTH places, so each of these statements appeared TWICE in
' examples/batch196_test.unimplemented.log.  Batch 196 keeps exactly one of the two.
'
' Verification (compile time, external): the log must list each statement once -
' 4 statements -> 4 lines.  This file is therefore COMPILE-ONLY for its assertion and
' prints nothing that a run would need to check.

FUNCTION PBMAIN() AS LONG
    ' INSTANCE myObj AS MyClass  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ' EVENTS Click, Changed  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ' EVENT SOURCE 1  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ' RAISEEVENT Click  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ConPrint "Batch 196 sample: nothing here depends on run-time behaviour."
' Press any key to exit...
ConWaitKey
END FUNCTION


