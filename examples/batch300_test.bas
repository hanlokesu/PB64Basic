#COMPILE EXE
#UNIQUE VAR ON
' Batch 300: A/B/C re-audit fixes - BIT CALC reset, ISNULL/ISNOTNULL,
' dual-compile: PBWin10 + PB64Basic; console via Win32 API (batch 234 pattern;
' PBWin /L /Q rejects bare PRINT with 477 and WAITKEY$ with 415).
DECLARE FUNCTION AllocConsole LIB "KERNEL32.DLL" ALIAS "AllocConsole" () AS LONG
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL s AS STRING)
    LOCAL h AS LONG
    LOCAL n AS DWORD
    h = GetStdHandle(-11)
    IF h = 0 THEN
        AllocConsole
        h = GetStdHandle(-11)
    END IF
    IF h <> 0 THEN
        WriteFile h, BYVAL STRPTR(s), LEN(s), n, 0
    END IF
END SUB

FASTPROC Add2 (BYVAL a AS LONG, BYVAL b AS LONG) AS LONG
END FASTPROC = a + b

#ALIGN 4
#COM NAME "Batch300", 1.0
#EXPORT Add2
#PAGE

FUNCTION PBMAIN() AS LONG
    LOCAL b AS LONG
    LOCAL s AS STRING
    LOCAL ok AS LONG
    b = 9
    BIT CALC b, 0, 0
    ConPrint "A=" & TRIM$(STR$(b)) & $CRLF    ' 8
    BIT CALC b, 4, 1
    ConPrint "C=" & TRIM$(STR$(b)) & $CRLF    ' 24
    BIT CALC b, 4, 0
    ConPrint "D=" & TRIM$(STR$(b)) & $CRLF    ' 8
    s = ""
    ConPrint "N=" & TRIM$(STR$(ISNULL(s))) & $CRLF      ' -1
    s = "x"
    ConPrint "NN=" & TRIM$(STR$(ISNOTNULL(s))) & $CRLF  ' -1
    ConPrint "FP=" & TRIM$(STR$(Add2(3, 4))) & $CRLF    ' 7
    ok = 1
    IF b <> 8 THEN ok = 0
    IF ISNULL("") <> -1 THEN ok = 0
    IF Add2(3, 4) <> 7 THEN ok = 0
    ConPrint "OK=" & TRIM$(STR$(ok)) & $CRLF
END FUNCTION
