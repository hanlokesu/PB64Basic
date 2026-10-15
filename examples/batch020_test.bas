#COMPILE EXE
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


' batch20_test.bas - ARRAY ARRAYIX / ARRAY SCAN / ARRAY INSERT / ARRAY DELETE / FILESCAN
' Status changes (batch 20): ARRAY SCAN/INSERT/DELETE promoted to IMPLEMENTED (verified),
' ARRAY ARRAYIX and FILESCAN newly implemented.
FUNCTION PBMAIN() AS LONG
    LOCAL waitk AS STRING
    DIM a(1 TO 5) AS LONG
    a(1) = 10
    a(2) = 20
    a(3) = 30
    a(4) = 25
    a(5) = 5
    ' ARRAY ARRAYIX: each element = its index
    ARRAY ARRAYIX a()
    ConPrint "ARRAYIX=" + STR$(a(1)) + "," + STR$(a(2)) + "," + STR$(a(3)) + "," + STR$(a(4)) + "," + STR$(a(5))
    ' reset values
    a(1) = 10
    a(2) = 20
    a(3) = 30
    a(4) = 25
    a(5) = 5
    ' ARRAY SCAN
    DIM idx AS LONG
    ARRAY SCAN a(), > 20, TO idx
    ConPrint "SCAN1=" + STR$(idx)
    ARRAY SCAN a(), = 25, TO idx
    ConPrint "SCAN2=" + STR$(idx)
    ' ARRAY INSERT / DELETE
    ARRAY DELETE a(2) FOR 1
    ConPrint "DEL=" + STR$(a(2)) + "," + STR$(a(3))
    ARRAY INSERT a(3), 99
    ConPrint "INS=" + STR$(a(3)) + "," + STR$(a(4))
    ' FILESCAN (INPUT mode)
    OPEN "b20_scan.txt" FOR OUTPUT AS #1
    PRINT #1, "line one"
    PRINT #1, "two"
    PRINT #1, "three words here"
    CLOSE #1
    DIM nrec AS LONG
    DIM wid AS LONG
    OPEN "b20_scan.txt" FOR INPUT AS #1
    FILESCAN #1, RECORDS TO nrec, WIDTH TO wid
    ConPrint "FSCAN=" + STR$(nrec) + "," + STR$(wid)
    CLOSE #1
    KILL "b20_scan.txt"
    FUNCTION = 0
    ConPrint "Press any key to exit..."
    waitk = WAITKEY$
END FUNCTION


#ENDIF
