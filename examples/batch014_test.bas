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

OPTION EXPLICIT

FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    DIM a() AS LONG
    DIM b() AS LONG
    DIM sa() AS STRING
    DIM s2() AS STRING
    DIM sb() AS STRING
    LOCAL i AS LONG
    LOCAL ip AS LONG
    LOCAL hostname AS STRING

    ' ---- ARRAY COPY (LONG) ----
    REDIM a(1 TO 5)
    REDIM b(1 TO 5)
    FOR i = 1 TO 5
        a(i) = i * 10
    NEXT i
    ARRAY COPY a(), b()
    IF b(1) <> 10 OR b(3) <> 30 OR b(5) <> 50 THEN
        ConPrint "COPY FAIL"
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    ' ---- ARRAY SWAP (LONG) ----
    REDIM a(1 TO 3)
    REDIM b(1 TO 3)
    a(1) = 1: a(2) = 2: a(3) = 3
    b(1) = 7: b(2) = 8: b(3) = 9
    ARRAY SWAP a(), b()
    IF a(1) <> 7 OR a(3) <> 9 OR b(2) <> 2 THEN
        ConPrint "SWAP FAIL"
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    ' ---- ARRAY COPY (STRING) ----
    REDIM s2(1 TO 3)
    REDIM sb(1 TO 3)
    s2(1) = "one": s2(2) = "two": s2(3) = "three"
    ARRAY COPY s2(), sb()
    IF sb(1) <> "one" OR sb(3) <> "three" THEN
        ConPrint "SCOPY FAIL"
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    ' ---- ARRAY UNIQUE (LONG) ----
    REDIM a(1 TO 6)
    a(1) = 1: a(2) = 2: a(3) = 2
    a(4) = 3: a(5) = 3: a(6) = 3
    ARRAY UNIQUE a()
    IF a(1) <> 1 OR a(2) <> 2 OR a(3) <> 3 THEN
        ConPrint "UNIQUE FAIL"
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    ' ---- ARRAY UNIQUE (STRING) ----
    REDIM sa(1 TO 4)
    sa(1) = "x": sa(2) = "y": sa(3) = "x": sa(4) = "z"
    ARRAY UNIQUE sa()
    IF sa(1) <> "x" OR sa(2) <> "y" OR sa(3) <> "z" THEN
        ConPrint "SUNIQUE FAIL"
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    ' ---- HOST ADDR ----
    HOST ADDR "localhost" TO ip
    IF ip = 0 THEN
        ConPrint "HOSTADDR FAIL"
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    ' ---- HOST NAME (ip 0 = local machine) ----
    HOST NAME 0 TO hostname
    IF LEN(hostname) = 0 THEN
        ConPrint "HOSTNAME FAIL"
        FUNCTION = 1
        EXIT FUNCTION
    END IF

    ConPrint "BATCH14 ALL PASS"
    FUNCTION = 0
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
