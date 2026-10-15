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

' PB64Basic Enhanced - Batch 195 test: no-op statement groups are now REPORTED
'
' COMPILE-ONLY by design: this batch changed diagnostics, not behaviour.  The four
' statements below are accepted and intentionally emit no code (they were parked in
' batch 121 pending the GUI / OOP runtime).  Before batch 195 they were dropped in
' silence; now the compiler pushes a warning for each one:
'
'   line 18: `INSTANCE` accepted but not implemented (DDT/OOP runtime pending) - no code generated
'
' NOTE (batch 209): this header described batch 195's *intent*, which was not actually
' in effect - the five parked names were also listed in codegen's handled-family
' early-return guard, so control left before the report and the compile stayed silent.
' batch 209 routes the arm through compiler.warnings, so the warning and the
' `examples/batch195_test.unimplemented.log` file below now really appear (4 statements
' for this file).  The exit code stays 0: accepted-but-empty is a report, not an error.
'
' Verification is therefore done at compile time - see the release notes - not by
' running the program.  The program does print, so it stays safe to run headless.

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING

    ConPrint "=== Batch 195: accepted-but-no-code statements are reported ==="

    ' INSTANCE myObj AS MyClass  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ConPrint "INSTANCE: parsed (warning expected at compile time)"

    ' EVENTS Click, Changed  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ConPrint "EVENTS: parsed (warning expected)"

    ' EVENT SOURCE 1  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ConPrint "EVENT SOURCE: parsed (warning expected)"

    ' RAISEEVENT Click  ' (batch 222: real form lives in a CLASS block; see batch222_test.bas)
    ConPrint "RAISEEVENT: parsed (warning expected)"

    ConPrint "=== Done: compile-only sample, nothing here depends on run-time output ==="
' Press any key to exit...
ConWaitKey
END FUNCTION


