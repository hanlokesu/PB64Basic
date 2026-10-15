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
' PB64Basic example - batch 186 test
'---------------------------------------------------------------------
' What this tests: ConPrint STR$(on the 32-bit target.)
'
' PRINT lowers to the CRT's printf.  UCRT keeps the legacy stdio names in
' legacy_stdio_definitions.lib - a library MSVC's own link line adds and
' clang's does not.  Before batch 186 this file therefore compiled for
' i686 but failed at link time:
'
'     lld-link: error: undefined symbol: _printf
'
' while the very same source linked fine for x64.  The compiler now
' locates that archive and passes it by full path in both link steps.
'
' scripts/link_smoke_32.py links every pbcompiler/tests/*.bas for both
' targets on each change, and pbcompiler/tests/l13_print_link.bas is the
' permanent guard; this example is the same case written for people
' rather than for the gate.
'
' Expected output (run mode):
'   batch 186 - PRINT links on both targets
'   6 * 7 = 42
'   done
' The second line used to arrive as two lines (`6 * 7 = ` then `42`); batch 187
' made the statement's trailing `;` suppress the newline, so this file's header
' was updated with it.
' Exit code: 0
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL answer AS LONG
    answer = 6 * 7
    ConPrint "batch 186 - PRINT links on both targets"
    ConPrint "6 * 7 = " & STR$(answer)
    ConPrint "done"
    FUNCTION = 0
' Press any key to exit...
ConWaitKey
END FUNCTION


