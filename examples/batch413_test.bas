#COMPILE EXE

'=====================================================================
' PB64Basic Enhanced - batch 413 regression test
'---------------------------------------------------------------------
' Purpose: pin the release identity for the batch that closed the
'          remaining Gary Beene common-dialog gaps.
'
'          The dialog behaviour itself is verified by the batch-4
'          screenshot harness (0773 font, 0775 open, 0777 save) because
'          a console test cannot capture a modal common dialog.  This
'          headless test keeps the packaged compiler and VERSION stamp
'          tied together and fails if the banner is not a v0.2 release.
'
' Expected output:
'   version -> v0.2.202
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

    ConPrint CHR$(13,10)
    ConPrint "=== FAILURES: " & STR$(fail) & " ===" & CHR$(13,10)
    FUNCTION = fail
END FUNCTION
