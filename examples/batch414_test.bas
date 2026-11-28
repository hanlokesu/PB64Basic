#COMPILE EXE

'=====================================================================
' PB64Basic Enhanced - batch 414 regression test
'---------------------------------------------------------------------
' Purpose: pin the release identity and the newly added COMCTL
'          constants from the fifth Gary Beene GUI batch.  The
'          visual fixes themselves are covered by the 50-sample
'          screenshot harness; this headless test fails if the
'          packaged compiler loses the constants or the banner.
'
' Expected output:
'   version -> v0.2.x (prefix check)
'   TCN_SELCHANGE -> -551
'   RGB_LIGHTGREEN -> 90EE90
'   === FAILURES: 0 ===
'   exit code 0
'=====================================================================

DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION AttachConsole LIB "KERNEL32.DLL" ALIAS "AttachConsole" (BYVAL dwProcessId AS DWORD) AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

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

FUNCTION PBMAIN () AS LONG
    LOCAL fail AS LONG
    LOCAL v AS STRING

    v = VERSION$
    ConPrint "version -> " & v & CHR$(13,10)
    IF LEFT$(v, 5) <> "v0.2." THEN fail = fail + 1
    IF LEN(v) <> 8 THEN fail = fail + 1

    ConPrint "TCN_SELCHANGE -> " & STR$(%TCN_SELCHANGE) & CHR$(13,10)
    IF %TCN_SELCHANGE <> -551 THEN fail = fail + 1

    ConPrint "RGB_LIGHTGREEN -> " & HEX$(%RGB_LIGHTGREEN) & CHR$(13,10)
    IF %RGB_LIGHTGREEN <> &H0090EE90 THEN fail = fail + 1

    ConPrint CHR$(13,10)
    ConPrint "=== FAILURES: " & STR$(fail) & " ===" & CHR$(13,10)
    FUNCTION = fail
END FUNCTION
