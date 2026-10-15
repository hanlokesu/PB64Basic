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

' PB64Basic Enhanced - batch220_test.bas
' batch 220: the remaining 125 guarded arms became loud (a short argument list
' is now a compile error, e.g. IMPORT ADDR without its TO AddrVar& target).
' This sample is the positive half of the witness: it drives a representative
' subset of those families with the argument counts the official pages require,
' and checks the calls really did something at run time.
FUNCTION PBMAIN () AS LONG
    LOCAL fails AS LONG
    LOCAL a AS LONG
    LOCAL f AS LONG
    LOCAL s AS STRING
    LOCAL mask AS STRING
    LOCAL w AS LONG, h AS LONG
    LOCAL x AS LONG, y AS LONG
    LOCAL ppix AS LONG, ppiy AS LONG
    LOCAL name1 AS STRING
    LOCAL name2 AS STRING

    ' --- MEMORY FILL (guard 3): write 4 bytes into a LONG, verify little-endian
    a = 0
    s = CHR$(1, 2, 3, 4)
    MEMORY FILL VARPTR(a), 4, s
    IF a = &H04030201 THEN
        ConPrint "OK: MEMORY FILL wrote the 4 bytes"
    ELSE
        ConPrint "FAIL: MEMORY FILL a=" & HEX$(a)
        INCR fails
    END IF

    ' --- NAME (guard 2): rename a file and confirm the result exists
    name1 = "batch220_name1.tmp"
    name2 = "batch220_name2.tmp"
    KILL name1
    KILL name2
    OPEN name1 FOR OUTPUT AS #1
    PRINT #1, "batch 220"
    CLOSE #1
    NAME name1 AS name2
    IF ISFILE(name2) AND ISFILE(name1) = 0 THEN
        ConPrint "OK: NAME renamed the file"
    ELSE
        ConPrint "FAIL: NAME rename did not land"
        INCR fails
    END IF

    ' --- PUT_STR / GET_STR (guards 2/3): binary ANSI string round-trip.
    '     PUT$ writes at the current file position and advances it, so rewind
    '     with SEEK # before reading the bytes back.
    OPEN name2 FOR BINARY AS #1
    PUT$ #1, "hello"
    SEEK #1, 1
    GET$ #1, 5, s
    CLOSE #1
    IF s = "hello" THEN
        ConPrint "OK: PUT/GET string round-trip"
    ELSE
        ConPrint "FAIL: PUT/GET s=[" & s & "]"
        INCR fails
    END IF
    KILL name2

    ' --- DIR family (guard 2): find the file we just deleted is gone, then
    '     list the batch test sources in this directory
    DIR "batch220_*.bas" TO s
    IF s = "" THEN
        ConPrint "OK: DIR returned empty for a deleted pattern"
    ELSE
        ConPrint "FAIL: DIR s=[" & s & "]"
        INCR fails
    END IF
    DIR CLOSE

    ' --- DESKTOP GET SIZE/CLIENT/LOC/PPI (guards 2): values must be sane
    DESKTOP GET SIZE TO w, h
    IF w > 0 AND h > 0 THEN
        ConPrint "OK: DESKTOP GET SIZE " & STR$(w) & "x" & STR$(h)
    ELSE
        ConPrint "FAIL: DESKTOP GET SIZE " & STR$(w) & "x" & STR$(h)
        INCR fails
    END IF
    DESKTOP GET CLIENT TO w, h
    IF w > 0 AND h > 0 THEN
        ConPrint "OK: DESKTOP GET CLIENT " & STR$(w) & "x" & STR$(h)
    ELSE
        ConPrint "FAIL: DESKTOP GET CLIENT " & STR$(w) & "x" & STR$(h)
        INCR fails
    END IF
    DESKTOP GET LOC TO x, y
    IF x >= 0 AND y >= 0 THEN
        ConPrint "OK: DESKTOP GET LOC " & STR$(x) & "," & STR$(y)
    ELSE
        ConPrint "FAIL: DESKTOP GET LOC " & STR$(x) & "," & STR$(y)
        INCR fails
    END IF
    DESKTOP GET PPI TO ppix, ppiy
    IF ppix > 0 AND ppiy > 0 THEN
        ConPrint "OK: DESKTOP GET PPI " & STR$(ppix) & "," & STR$(ppiy)
    ELSE
        ConPrint "FAIL: DESKTOP GET PPI " & STR$(ppix) & "," & STR$(ppiy)
        INCR fails
    END IF

    IF fails = 0 THEN
        ConPrint "batch220: ALL PASS"
    ELSE
        ConPrint "batch220: FAILURES=" & STR$(fails)
    END IF
    ConPrint "=== FAILURES: " & STR$(fails) & " ==="

' waiting for any key to exit...
WAITKEY$
END FUNCTION


#ENDIF
