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
' PB64Basic example - batch 187 test
'---------------------------------------------------------------------
' What this tests: a trailing `;` (or `,`) on PRINT suppresses the newline.
'
' PowerBASIC's rule, quoted from the manual and repeated in the official corpus:
' a semicolon at the end of a PRINT statement leaves the cursor where it is, so
' the next PRINT continues the same line.  Until batch 187 this compiler printed
' the newline anyway:
'
'     PRINT "6 * 7 = "        <- no separator: its own line
'     PRINT 6 * 7
'
' produced
'
'     6 * 7 =
'     42
'
' where PowerBASIC produces `6 * 7 = 42`.
'
' The separator is now carried on the statement (PrintStmt::trailing) instead of
' being dropped by the parser, and the code generator emits the newline only when
' there was none.
'
' Known divergence, stated rather than hidden: a trailing COMMA suppresses the
' newline exactly like a semicolon here, but PowerBASIC also advances the cursor
' to the next print zone (14 columns).  Zone advance needs the current column, so
' it is not implemented yet.
'
' Expected output (run mode) - four lines, the first two joined by the `;`:
'   6 * 7 = 42
'   left/right
'   one
'   two
' Exit code: 0
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL answer AS LONG
    answer = 6 * 7
    ConPrint "6 * 7 = " & STR$(answer)
    ConPrint "left" & "/" & "right"
    ConPrint "one"
    ConPrint "two"
    FUNCTION = 0
' Press any key to exit...
ConWaitKey
END FUNCTION


