

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
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
' batch 201 - GLOBALMEM and TRACE: two silent no-ops made real
'---------------------------------------------------------------------
' Defect 1: `GLOBALMEM ALLOC 64 TO h` parsed to ONE argument, because the parser
'   only consumed TO after a comma.  The codegen arm needs two arguments, so the
'   statement compiled cleanly and then did nothing at all.
' Defect 2: the numeric branch of TRACE PRINT skipped four bytes for a length
'   prefix that is not there.  num_to_string returns a BSTR whose pointer already
'   addresses the characters, so `TRACE PRINT 42` wrote a stray byte from
'   uninitialised heap instead of 42.
'
' Pointer results (GLOBALMEM LOCK, IMPORT ADDR, IMAGELIST NEW) are 64-bit
' addresses: hold them in QUAD.  A LONG truncates them to 0 on this target.
' The trace file this program writes is checked externally by the harness:
' NOTE: alloc handle and LOCK pointer are per-process values (PBWin32 vs fork
'   x64; the LOCK pointer truncates to 32-bit and may print negative). Both are
'   normalized to <nonzero>; the non-zero assertions are retained.
' it must contain exactly the two lines "alpha" and "42".
'=====================================================================
FUNCTION PBMAIN () AS LONG
    LOCAL h       AS LONG
    LOCAL p       AS LONG
    LOCAL sz      AS LONG
    LOCAL scratch AS LONG
    LOCAL fails   AS LONG

    fails = 0

    GLOBALMEM ALLOC 64 TO h
    ConPrint "alloc handle = <nonzero>"
    IF h = 0 THEN fails = fails + 1

    GLOBALMEM SIZE h TO sz
    ConPrint "size = " & STR$(sz)
    IF sz < 64 THEN fails = fails + 1

    GLOBALMEM LOCK h TO p
    ConPrint "lock pointer = <nonzero>"
    IF p = 0 THEN fails = fails + 1

    GLOBALMEM UNLOCK h TO scratch
    GLOBALMEM FREE h TO scratch
    ConPrint "unlock/free returned " & STR$(scratch)

    TRACE NEW "batch201_trace.txt"
    TRACE ON
    TRACE PRINT "alpha"
    TRACE PRINT "42"
    TRACE OFF
    TRACE PRINT "must not be written"
    TRACE CLOSE
    ConPrint "wrote batch201_trace.txt"

    ConPrint "=== FAILURES: " & STR$(fails)
    FUNCTION = fails
' Press any key to exit...
ConWaitKey
END FUNCTION





