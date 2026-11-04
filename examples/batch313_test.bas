#COMPILE EXE
#DIM ALL

' PB64Basic Enhanced - batch 313 test
' Three 32-bit codegen fixes: object compilation now uses
' -ffunction-sections -fdata-sections so the linker keeps one main/PBMAIN body;
' no-LIB DECLARE on i686 uses the default stdcall convention and the decorated
' symbol; and DECLARE ... LIB treats user DLLs as stdcall on i686. The full
' stdcall/DLL fidelity is covered by Gate 6; this is a standalone headless smoke
' that exercises the multi-function codegen path and verifies results. It writes
' with GetStdHandle/WriteFile and has no WAITKEY/MSGBOX tail.
DECLARE FUNCTION GetStdHandle LIB "KERNEL32.DLL" ALIAS "GetStdHandle" (BYVAL nStdHandle AS DWORD) AS LONG
DECLARE FUNCTION WriteFile LIB "KERNEL32.DLL" ALIAS "WriteFile" (BYVAL hFile AS LONG, lpBuffer AS ANY, BYVAL nBytesToWrite AS DWORD, lpBytesWritten AS DWORD, BYVAL lpOverlapped AS LONG) AS LONG

SUB ConPrint(BYVAL h AS LONG, BYVAL t AS STRING)
    LOCAL w AS DWORD
    WriteFile h, BYVAL STRPTR(t), LEN(t), w, 0
END SUB

FUNCTION Square(BYVAL n AS LONG) AS LONG
    FUNCTION = n * n
END FUNCTION

FUNCTION PBMAIN() AS LONG
    LOCAL h AS LONG
    LOCAL fails AS LONG
    LOCAL i AS LONG
    LOCAL s AS LONG
    h = GetStdHandle(-11)

    ' Multi-function path: a separate Square function is called in a loop.
    FOR i = 1 TO 4
        s = Square(i)
        ConPrint h, "sq " & TRIM$(i) & " " & TRIM$(s) & $CRLF
    NEXT i
    IF Square(7) <> 49 THEN fails = fails + 1

    ' Basic arithmetic still evaluates correctly after the codegen changes.
    IF 6 * 7 <> 42 THEN fails = fails + 1
    IF 20 + 23 <> 43 THEN fails = fails + 1

    ConPrint h, "FAILS=" & TRIM$(fails) & $CRLF
    IF fails = 0 THEN ConPrint h, "batch313 test ok" & $CRLF
    FUNCTION = 0
END FUNCTION