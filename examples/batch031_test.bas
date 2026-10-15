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

' PB64Basic Enhanced - batch 31 test: FIELD + RANDOM record I/O
' Covers: OPEN FOR RANDOM LEN=, FIELD #n size AS var,
'         PUT/GET record (current + numbered), blank padding,
'         FIELD dyn$, FIELD RESET, FIELD STRING.
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL f1 AS FIELD, f2 AS FIELD
    LOCAL g1 AS FIELD, g2 AS FIELD
    LOCAL x$ AS STRING
    LOCAL errc AS LONG

    ' --- 1) RANDOM file + field binding + PUT record ---
    KILL "rec31.dat"
    OPEN "rec31.dat" FOR RANDOM AS #1 LEN=20
    FIELD #1, 10 AS f1, 10 AS f2
    f1 = "0123456789"
    f2 = "9876543210"
    PUT #1
    CLOSE #1

    ' --- 2) Re-open, read record back, verify fields ---
    OPEN "rec31.dat" FOR RANDOM AS #2 LEN=20
    FIELD #2, 10 AS f1, 10 AS f2
    GET #2
    IF f1 <> "0123456789" THEN
        ConPrint "FAIL f1 read = [" & STR$(f1) & "]"
        errc = errc + 1
    END IF
    IF f2 <> "9876543210" THEN
        ConPrint "FAIL f2 read = [" & STR$(f2) & "]"
        errc = errc + 1
    END IF

    ' --- 3) Shorter assignment pads with blanks ---
    FIELD #2, 10 AS f1
    f1 = "abc"
    IF f1 <> "abc       " THEN
        ConPrint "FAIL pad = [" & STR$(f1) & "]"
        errc = errc + 1
    END IF

    ' --- 4) Numbered records: write record 2, read it back ---
    FIELD #2, 10 AS f1, 10 AS f2
    f1 = "RECORDTWO"
    f2 = "1234567890"
    PUT #2, 2
    GET #2, 2
    IF f1 <> "RECORDTWO " THEN
        ConPrint "FAIL rec2 f1 = [" & STR$(f1) & "]"
        errc = errc + 1
    END IF
    IF f2 <> "1234567890" THEN
        ConPrint "FAIL rec2 f2 = [" & STR$(f2) & "]"
        errc = errc + 1
    END IF

    ' --- 6) FIELD STRING keeps a private copy (must happen while the
    '        record buffer is still open; after CLOSE the binding is gone) ---
    FIELD STRING f1
    IF f1 <> "RECORDTWO " THEN
        ConPrint "FAIL tostr f1 = [" & STR$(f1) & "]"
        errc = errc + 1
    END IF
    CLOSE #2

    ' --- 5) Dynamic string binding ---
    FIELD x$, 3 AS g1, 3 AS g2
    x$ = "abcdef"
    IF g1 <> "abc" THEN
        ConPrint "FAIL dyn g1 = [" & STR$(g1) & "]"
        errc = errc + 1
    END IF
    IF g2 <> "def" THEN
        ConPrint "FAIL dyn g2 = [" & STR$(g2) & "]"
        errc = errc + 1
    END IF
    g1 = "111"
    g2 = "222"
    IF x$ <> "111222" THEN
        ConPrint "FAIL dyn write x$ = [" & x$ & "]"
        errc = errc + 1
    END IF

    ' --- 7) FIELD RESET unbinds ---
    FIELD RESET g1
    IF g1 <> "" THEN
        ConPrint "FAIL reset g1 = [" & STR$(g1) & "]"
        errc = errc + 1
    END IF

    KILL "rec31.dat"

    IF errc = 0 THEN
        ConPrint "ALL PASS (7 groups)"
    ELSE
        ConPrint "FAILURES: " & STR$(errc)
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
