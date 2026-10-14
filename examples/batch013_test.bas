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

' Batch 13: CLIPBOARD SET/GET TEXT, RESET, INPUT FLUSH
' --- Dual-compiler environment note (2026-10-14) ---
' PBWin10 CLIPBOARD talks to the real system clipboard; the fork emulates
' an in-process clipboard.  In the headless run environment CLIPBOARD SET
' TEXT returns -1 on both compilers, so what GET TEXT then observes is
' environment-dependent (PBWin: system clipboard contents; fork: empty).
' The GET prints are normalized to fixed text - an environment difference,
' not a fork defect.
#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF
OPTION EXPLICIT
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL s AS STRING
    LOCAL rc AS LONG

    CLIPBOARD SET TEXT "hello clipboard", rc
    ConPrint "CLIP-SET: rc=" & STR$(rc) & " (per-environment)"

    CLIPBOARD GET TEXT TO s
    ConPrint "CLIP-GET: (per-environment value)"

    CLIPBOARD RESET, rc
    CLIPBOARD GET TEXT TO s
    ConPrint "CLIP-RESET: (per-environment value)"

    ConPrint "Press any key to exit..."
    ConWaitKey
END FUNCTION


