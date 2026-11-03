#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 311 test
' Fix: exact PowerBASIC numeric rendering and a clean L-family dual-run.
' The runtime number renderer now matches PBWin's x87-style 15-digit
' half-even formatting, so LOG/LOG2/LOG10 print byte-identical output, and
' LOWRD is verified through a ConPrint probe instead of a bare PRINT that
' PBWin rejects with Error 477.
' Headless smoke: it writes verifiable lines with GetStdHandle/WriteFile and
' checks each expected value; there is no WAITKEY/MSGBOX tail.
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL h AS LONG, BYVAL t AS STRING)
    LOCAL w AS DWORD
    WriteFile h, BYVAL STRPTR(t), LEN(t), w, 0
END SUB

FUNCTION PBMAIN() AS LONG
    LOCAL h AS LONG
    LOCAL x AS DWORD
    LOCAL fails AS LONG
    h = GetStdHandle(-11)

    ' Logarithm family: the default 15-digit renderer must match PBWin exactly.
    ' (TRIM$ uses the STR$ rendering path; FORMAT$ with no mask uses a different
    ' higher-precision path, so the verified dual-run values are compared here.)
    ConPrint h, "LOG(10)=" & TRIM$(LOG(10)) & $CRLF
    ConPrint h, "LOG2(8)=" & TRIM$(LOG2(8)) & $CRLF
    ConPrint h, "LOG10(1000)=" & TRIM$(LOG10(1000)) & $CRLF

    IF TRIM$(LOG(10)) <> "2.30258509299404" THEN fails = fails + 1
    IF TRIM$(LOG2(8)) <> "3" THEN fails = fails + 1
    IF TRIM$(LOG10(1000)) <> "3" THEN fails = fails + 1

    ' LOWRD: low 16-bit word of a 32-bit value.
    x = &H12345678
    ConPrint h, "LOWRD(12345678)=" & TRIM$(LOWRD(x)) & $CRLF
    IF LOWRD(x) <> 22136 THEN fails = fails + 1
    x = &HABCD0001
    ConPrint h, "LOWRD(ABCD0001)=" & TRIM$(LOWRD(x)) & $CRLF
    IF LOWRD(x) <> 1 THEN fails = fails + 1

    ConPrint h, "FAILS=" & TRIM$(fails) & $CRLF
    IF fails = 0 THEN ConPrint h, "batch311 test ok" & $CRLF
    FUNCTION = 0
END FUNCTION
