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



' =====================================================================

' batch212_test.bas - witness for "CLASS ... INSTANCE" real storage

' ---------------------------------------------------------------------

' What this asserts, and nothing else:

'   * a CLASS whose INSTANCE lines declare members now produces a real

'     object type, so two variables of that type keep INDEPENDENT values

'   * each member keeps its own slot (x and y do not alias)

'   * a member can be read back after a whole-object copy

' The official CLASS page shows exactly this shape:

'   CLASS name / INSTANCE var AS type / END CLASS

' =====================================================================

CLASS Point

    INSTANCE x AS LONG

    INSTANCE y AS LONG

END CLASS



FUNCTION PBMAIN() AS LONG

    LOCAL p AS Point

    LOCAL q AS Point

    LOCAL fails AS LONG

    LOCAL t AS Point



    fails = 0

    p.x = 11

    p.y = 22

    q.x = 33

    q.y = 44



    ConPrint "p = " & STR$(p.x) & "," & STR$(p.y)
    ConPrint "q = " & STR$(q.x) & "," & STR$(q.y)


    IF p.x <> 11 THEN fails = fails + 1

    IF p.y <> 22 THEN fails = fails + 1

    IF q.x <> 33 THEN fails = fails + 1

    IF q.y <> 44 THEN fails = fails + 1

    ' per-object independence: q.x must not have touched p.x

    IF p.x = q.x THEN fails = fails + 1

    ' member slots do not alias inside one object

    IF p.x = p.y THEN fails = fails + 1



    t = p

    ConPrint "t = " & STR$(t.x) & "," & STR$(t.y)
    IF t.y <> 22 THEN fails = fails + 1



    ConPrint "=== FAILURES:" & STR$(fails) & "==="
    FUNCTION = 0

' Press any key to exit...
WAITKEY$
END FUNCTION



#ENDIF
