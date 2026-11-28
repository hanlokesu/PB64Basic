' =====================================================================
' PB64Basic Enhanced - batch 424 - EXT precision inside THREAD CREATE
'   Windows starts each thread with the default x87 control word
'   (53-bit significand).  The runtime's thread trampoline must raise it
'   to 0x037F before the PB procedure runs, otherwise EXT division and
'   SQR silently degrade to double.
'   === FAILURES: 0
' =====================================================================
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
    IF h <> 0 THEN WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
END SUB

GLOBAL gs AS STRING

FUNCTION Worker () AS LONG
    LOCAL e AS EXT
    e = 1
    e = e / 3
    gs = STR$(e, 18) & "|" & STR$(SQR(2), 18)
    FUNCTION = 0
END FUNCTION

FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    fails = 0
    THREAD CREATE Worker
    SLEEP 500
    ConPrint gs & CHR$(13, 10)
    IF gs <> " .333333333333333333| 1.41421356237309505" THEN fails = fails + 1
    ConPrint "=== FAILURES: " & STR$(fails) & CHR$(13, 10)
    FUNCTION = 0
END FUNCTION
