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

' Batch 2: SWAP / SHIFT / ROTATE / ARRAY REVERSE / PUT$
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    LOCAL a AS LONG
    LOCAL b AS LONG
    LOCAL n AS LONG
    LOCAL i AS LONG
    LOCAL f AS LONG
    LOCAL s AS STRING
    LOCAL arr() AS LONG

    a = 1
    b = 2
    SWAP a, b
    IF a = 2 AND b = 1 THEN
        ConPrint "SWAP-PASS"
    ELSE
        ConPrint "SWAP-FAIL " & STR$(a) & STR$(b)
    END IF

    n = 221
    SHIFT LEFT n, 1
    IF n = 442 THEN
        ConPrint "SHIFTLEFT-PASS"
    ELSE
        ConPrint "SHIFTLEFT-FAIL " & STR$(n)
    END IF
    n = 221
    SHIFT RIGHT n, 1
    IF n = 110 THEN
        ConPrint "SHIFTRIGHT-PASS"
    ELSE
        ConPrint "SHIFTRIGHT-FAIL " & STR$(n)
    END IF
    n = -4
    SHIFT SIGNED RIGHT n, 1
    IF n = -2 THEN
        ConPrint "SHIFTSIGNED-PASS"
    ELSE
        ConPrint "SHIFTSIGNED-FAIL " & STR$(n)
    END IF
    n = 1
    ROTATE LEFT n, 1
    IF n = 2 THEN
        ConPrint "ROTATELEFT-PASS"
    ELSE
        ConPrint "ROTATELEFT-FAIL " & STR$(n)
    END IF
    n = 2
    ROTATE RIGHT n, 1
    IF n = 1 THEN
        ConPrint "ROTATERIGHT-PASS"
    ELSE
        ConPrint "ROTATERIGHT-FAIL " & STR$(n)
    END IF

    REDIM arr(1 TO 5)
    FOR i = 1 TO 5
        arr(i) = i
    NEXT i
    ARRAY REVERSE arr()
    IF arr(1) = 5 AND arr(3) = 3 AND arr(5) = 1 THEN
        ConPrint "ARRAYREV-PASS"
    ELSE
        ConPrint "ARRAYREV-FAIL " & STR$(arr(1)) & STR$(arr(3)) & STR$(arr(5))
    END IF

    f = FREEFILE
    OPEN "putstr_test.dat" FOR BINARY AS #f
    PUT$ #f, "ABC"
    PUT$ #f, "123"
    CLOSE #f
    OPEN "putstr_test.dat" FOR INPUT AS #f
    LINE INPUT #f, s
    CLOSE #f
    KILL "putstr_test.dat"
    IF s = "ABC123" THEN
        ConPrint "PUT$-PASS"
    ELSE
        ConPrint "PUT$-FAIL [" & s & "]"
    END IF
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
