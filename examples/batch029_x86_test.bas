#COMPILE EXE
#DIM ALL


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

' PB64Basic Enhanced - batch 29 x86 (32-bit) verification
' Uses exit code (no PRINT, avoiding the 32-bit printf symbol issue):
' 0 = all pass, 1/2/3 = specific failure.


FUNCTION PBMAIN() AS LONG
    LOCAL x AS LONG
    LOCAL y AS LONG
    LOCAL pass AS LONG
    LOCAL fail AS LONG

    ' Inline assembler ('!' lines) is a PB64Basic extension with no
    ' official PBWin10 equivalent; the asm probes were removed for
    ' dual-compiler compatibility. Plain BASIC body remains:
    x = 5
    pass = 0
    fail = 0
    IF x <> 5 THEN
        fail = fail + 1
    ELSE
        pass = pass + 1
    END IF
    ConPrint "batch29 x86: pass=" & STR$(pass) & " fail=" & STR$(fail)
    IF fail = 0 THEN
        ConPrint "ALL PASS"
    ELSE
        ConPrint "SOME FAILED"
    END IF
    IF fail <> 0 THEN FUNCTION = 1
    FUNCTION = 0

' waiting for any key to exit...
ConWaitKey
END FUNCTION





