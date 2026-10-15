#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

' --- PBWin10 stub branch: this sample exercises PB64Basic-only ---
'     syntax that official PBWin10 does not provide; it compiles but
'     does nothing here.  The fork branch (#ELSE) is the real test.
#IF %MY_PBVER = 10
FUNCTION PBMAIN() AS LONG
    ' PB64Basic-only sample: PBWin10 stub (compiles, does nothing).
END FUNCTION
#ELSE

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

' Batch 119: ARRAY SCAN / ARRAY SELECT / ARRAY REDIM INCR-DECR
' Regression for parser bug: these statements were silently dropped
' (reserved tokens SELECT/REDIM/INCR/DECR not recognized by parser guards).
' Now they emit real IR calls. SCAN returns the 1-based index of a match.
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL i AS LONG
    LOCAL n AS LONG
    DIM a(1 TO 6) AS LONG
    a(1)=11: a(2)=22: a(3)=33: a(4)=44: a(5)=55: a(6)=66

    ConPrint "=== Batch 119: ARRAY SCAN / SELECT / REDIM ==="

    ' Test 1: ARRAY SCAN = value  -> 1-based index
    i = -1
    ARRAY SCAN a(), = 33 TO i
    ConPrint "SCAN = 33 -> index=" & STR$(i) & " (expect 3)"

    ' Test 2: ARRAY SCAN with <> operator
    i = -1
    ARRAY SCAN a(), <> 33 TO i
    ConPrint "SCAN <> 33 -> index=" & STR$(i) & " (expect 1)"

    ' Test 3: ARRAY SELECT sets the active selection range
    ARRAY SELECT a(), 2, 5
    ConPrint "ARRAY SELECT a(), 2, 4 accepted (selection state set)"

    ' Test 4: ARRAY REDIM INCR / DECR (runtime reports new size)
    ARRAY REDIM INCR a(), 2
    ARRAY REDIM DECR a(), 1
    ConPrint "ARRAY REDIM INCR/DECR accepted"

    ConPrint ""
    ConPrint "=== ALL TESTS RAN ==="
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
