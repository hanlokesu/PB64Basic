#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 314 test
' The packaged runtime object is now a function-sections build (988 .text
' sections, 3,324 COMDAT) so /OPT:REF drops unused runtime routines; the
' release verifier gates on the COMDAT count. This is a standalone headless
' smoke that exercises a multi-function program and checks results. It uses
' GetStdHandle/WriteFile and has no WAITKEY/MSGBOX tail.
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL h AS LONG, BYVAL t AS STRING)
    LOCAL w AS DWORD
    WriteFile h, BYVAL STRPTR(t), LEN(t), w, 0
END SUB

FUNCTION Add2(BYVAL a AS LONG, BYVAL b AS LONG) AS LONG
    FUNCTION = a + b
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL h AS LONG
    LOCAL fails AS LONG
    h = GetStdHandle(-11)

    IF Add2(3, 4) <> 7 THEN fails = fails + 1
    IF Add2(-5, 20) <> 15 THEN fails = fails + 1
    IF 8 * 8 <> 64 THEN fails = fails + 1

    ConPrint h, "FAILS=" & TRIM$(fails) & $CRLF
    IF fails = 0 THEN ConPrint h, "batch314 test ok" & $CRLF
    FUNCTION = 0
END FUNCTION
