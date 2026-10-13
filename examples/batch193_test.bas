

' --- PBWin10 stub branch: this sample exercises PowerBasilisk-only ---
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

'=====================================================================
' batch193_test.bas - batch 193: 21 silent Noop sites now hard-fail
'---------------------------------------------------------------------
' Batch 193 turned the last 21 silently-dropped parser sites into loud compile
' errors (14 -> 3 silent sites, the three kept on purpose: bare TYPE at module
' level, #INCLUDE, one commented-out site).
'
' A hard error cannot appear inside a compiling sample, so this file is the
' *positive control*: the statements that share those code paths - a LET, a
' module-level UDT, METRICS, CHR$ and an ImageList handle - must still compile
' and still work.  If the batch had been over-tightened, one of these breaks and
' this sample stops building, which is exactly the signal we want.
'
' Asserts: LET binds, the UDT round-trips, METRICS is positive, CHR$ works and
' the ImageList handle arrives.
'=====================================================================
TYPE Pt
    x AS LONG
    y AS LONG
END TYPE

#IF %DEF(%PB_REVISION)
    #IF (%PB_REVISION AND &H0FF00) = &H1000
        %MY_PBVER = 10
    #ELSE
        %MY_PBVER = 0
    #ENDIF
#ELSE
    %MY_PBVER = 0
#ENDIF

FUNCTION PBMAIN () AS LONG
    LOCAL fail AS LONG
    LOCAL v AS LONG
    LOCAL p AS Pt
    #IF %MY_PBVER = 0
        LOCAL hil AS QUAD
    #ELSE
        LOCAL hil AS LONG
    #ENDIF
    LOCAL cnt AS LONG
    LET v = 41
    v = v + 1
    ConPrint "LET 41+1     = " & STR$(v)
    IF v <> 42 THEN
        fail = fail + 1
    END IF
    p.x = 3
    p.y = 4
    ConPrint "UDT point    = " & STR$(p.x) & " $TAB " & STR$(p.y)
    IF p.x <> 3 THEN
        fail = fail + 1
    END IF
    IF METRICS(Scroll.Horz) <= 0 THEN
        fail = fail + 1
    END IF
    IF CHR$(65) <> "A" THEN
        fail = fail + 1
    END IF
    IMAGELIST NEW BITMAP 16, 16, 32, 2 TO hil
    IMAGELIST GET COUNT hil TO cnt
    ConPrint "imagelist hi = " & STR$(hil) & "  count = " & STR$(cnt)
    IF hil = 0 THEN
        fail = fail + 1
    END IF
    IF cnt <> 0 THEN
        fail = fail + 1
    END IF
    IMAGELIST KILL hil
    ConPrint "=== FAILURES:" & STR$(fail) & "==="
    FUNCTION = fail
' Press any key to exit...
ConWaitKey
END FUNCTION





