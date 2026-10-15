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

'=====================================================================
' PBXB64 Example - batch 214 witness: CLASS with a real METHOD block
'---------------------------------------------------------------------
' Demonstrates:
'   * METHOD Name [(params)] [AS type] ... END METHOD inside CLASS
'   * the method leaves the parser as an ordinary FUNCTION named
'     <Class>_<Method> (here Point_Twice) whose first parameter is the
'     implicit BYREF receiver added in batch 216, so the call passes the
'     object: got = Point_Twice(p, 21)
'   * END METHOD is a real block terminator and the return type is
'     honoured (before batch 214 the CLASS body was skipped line by line
'     and a METHOD body ran on to the end of the file)
' Expected output (run mode):
'   twice =42
'   === FAILURES:0 ===
' Complexity: O(1) - one multiply, one console write.
'=====================================================================

CLASS Point
    INSTANCE x AS LONG
    METHOD Twice(v AS LONG) AS LONG
        FUNCTION = v * 2
    END METHOD
END CLASS

FUNCTION PBMAIN() AS LONG
    LOCAL got AS LONG
    LOCAL p AS Point
    got = Point_Twice(p, 21)
    ConPrint "twice =" & STR$(got)
    IF got = 42 THEN
        ConPrint "=== FAILURES:0 ==="
    ELSE
        ConPrint "=== FAILURES:1 ==="
    END IF
    FUNCTION = 0
' Press any key to exit...
WAITKEY$
END FUNCTION


#ENDIF
