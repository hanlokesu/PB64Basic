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

' PB64Basic Enhanced - batch219_test.bas
' batch 219: the statements whose argument-count guard became loud.
' This sample drives two of them (FILECOPY, SETATTR) with the argument counts
' the official help pages require - the positive half of the batch's witness.
' A short argument list is a compile error now (see _b219_probes/*.bas), so this
' file must keep using the full forms.
FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL src AS STRING
    LOCAL dst AS STRING
    LOCAL e AS LONG

    src = "batch219_src.tmp"
    dst = "batch219_dst.tmp"

    OPEN src FOR OUTPUT AS #1
    PRINT #1, "batch 219"
    CLOSE #1

    FILECOPY src, dst
    e = ERR
    IF e = 0 AND ISFILE(dst) THEN
        ConPrint "OK: FILECOPY copied the file"
    ELSE
        ConPrint "FAIL: FILECOPY err=" & STR$(e)
        INCR fails
    END IF

    SETATTR dst, 1
    e = ERR
    IF e = 0 AND ISFILE(dst) THEN
        ConPrint "OK: SETATTR accepted the attribute"
    ELSE
        ConPrint "FAIL: SETATTR err=" & STR$(e)
        INCR fails
    END IF

    SETATTR dst, 0
    IF ERR = 0 THEN ConPrint "OK: SETATTR cleared the attribute"

    KILL src
    KILL dst
    IF ISFILE(src) = 0 AND ISFILE(dst) = 0 THEN
        ConPrint "OK: temporary files removed"
    ELSE
        ConPrint "FAIL: temporary files left behind"
        INCR fails
    END IF

    IF fails = 0 THEN
        ConPrint "batch219: ALL PASS"
    ELSE
        ConPrint "batch219: FAILURES=" & STR$(fails)
    END IF
    ConPrint "=== FAILURES: " & STR$(fails) & " ==="
' Press any key to exit...
ConWaitKey
END FUNCTION


